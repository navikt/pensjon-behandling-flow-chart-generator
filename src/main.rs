use anyhow::{Context, Result};
use clap::Parser as ClapParser;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tree_sitter::Parser;
use walkdir::WalkDir;

/// Analyze and visualize Kotlin Behandling flow graphs
#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Kotlin project directory (defaults to current directory)
    #[arg(value_name = "PATH")]
    path: Option<String>,

    /// Output format for the graph (svg, png, pdf, etc.)
    #[arg(short, long, default_value = "svg")]
    format: String,

    /// Edge style: curved, straight, or ortho (orthogonal)
    #[arg(short = 'e', long, default_value = "straight")]
    edge_style: String,

    /// Show condition labels on edges (default: hidden for cleaner graphs)
    #[arg(short = 'c', long)]
    show_conditions: bool,

    /// Show color legend in graph (default: hidden)
    #[arg(short = 'l', long)]
    show_legend: bool,

    /// Automatically open the generated graph
    #[arg(long)]
    open: bool,

    /// Keep the intermediate .dot file
    #[arg(short, long)]
    keep_dot: bool,

    /// Output directory for generated files (defaults to current directory)
    #[arg(short, long)]
    output_dir: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable edge deduplication and consolidation (shows all raw edges)
    #[arg(long)]
    no_deduplicate: bool,
}

#[derive(Debug, Clone)]
struct ClassInfo {
    name: String,
    file: PathBuf,
    supertypes: Vec<String>,
    initial_aktivitet: Option<String>,
}

#[derive(Debug, Clone)]
struct ProcessorInfo {
    processor_class: String,
    next_aktiviteter: Vec<NextAktivitet>,
    has_manuell_behandling: bool,
}

#[derive(Debug, Clone)]
struct NextAktivitet {
    aktivitet_name: String,
    condition: Option<String>,
    is_collection: bool, // True if this represents multiple instances (fan-out)
}

#[derive(Debug, Clone)]
struct IterationGroup {
    trigger_node: String,        // Node that starts the iteration
    iterated_nodes: Vec<String>, // All nodes that are part of the iteration path
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Edge {
    from: String,
    to: String,
    label: String,
    is_collection: bool, // True if this represents multiple instances (fan-out)
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Use provided path or current directory
    let root_folder = args.path.unwrap_or_else(|| ".".to_string());

    // Validate that the path exists
    let root_path = PathBuf::from(&root_folder);
    if !root_path.exists() {
        anyhow::bail!("Path does not exist: {}", root_folder);
    }
    if !root_path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", root_folder);
    }

    println!("🔍 Scanning directory: {}", root_folder);

    // 2. Initialize Tree-sitter Kotlin parser
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_kotlin::language())
        .context("Failed to set Kotlin language")?;

    // 3. Walk all subfolders and collect .kt files
    let kt_files = collect_kotlin_files(&root_folder)?;
    if kt_files.is_empty() {
        anyhow::bail!("No .kt files found in directory: {}", root_folder);
    }
    println!("📄 Scanned {} .kt files", kt_files.len());

    // 4. Build a class index
    let class_index = build_class_index(&mut parser, &kt_files)?;
    println!("📚 Indexed {} classes", class_index.len());

    // 4.5. Build processor index
    let processor_index = build_processor_index(&mut parser, &kt_files)?;
    println!("⚙️  Found {} processors", processor_index.len());

    if args.verbose {
        println!("\n=== PROCESSOR DETAILS ===");
        let mut processors: Vec<_> = processor_index.iter().collect();
        processors.sort_by(|a, b| a.0.cmp(b.0));
        for (aktivitet, info) in processors {
            println!("\n  {} (handled by {})", aktivitet, info.processor_class);
            if info.has_manuell_behandling {
                println!("    📋 Creates manuellBehandling");
            }
            if info.next_aktiviteter.is_empty() {
                println!("    → [END]");
            } else {
                for next in &info.next_aktiviteter {
                    if let Some(condition) = &next.condition {
                        println!("    → [{}] {}", condition, next.aktivitet_name);
                    } else {
                        println!("    → {}", next.aktivitet_name);
                    }
                }
            }
        }
    }

    // 5. Print basic debug info (only in verbose mode)
    if args.verbose {
        println!("\n=== SUMMARY ===");
    }

    // Find main Behandling classes (ones with initial aktivitet)
    let mut main_behandling_classes: Vec<_> = class_index
        .iter()
        .filter(|(_, info)| {
            info.supertypes.iter().any(|s| s.contains("Behandling"))
                && info.initial_aktivitet.is_some()
        })
        .collect();

    main_behandling_classes.sort_by(|a, b| a.0.cmp(b.0));

    if !main_behandling_classes.is_empty() {
        if args.verbose {
            println!("\nMain Behandling classes with initial aktivitet:");
            for (name, info) in &main_behandling_classes {
                println!(
                    "\n  {} ({})",
                    name,
                    info.file.file_name().unwrap().to_string_lossy()
                );
                if let Some(initial) = &info.initial_aktivitet {
                    println!("    → opprettInitiellAktivitet() returns: {}", initial);
                }
            }
        }
    } else {
        anyhow::bail!("No Behandling classes with initial aktivitet found!");
    }

    if args.verbose {
        println!("\n\n=== ALL BEHANDLING CLASSES ===");
        let mut all_behandling: Vec<_> = class_index
            .iter()
            .filter(|(_, info)| info.supertypes.iter().any(|s| s.contains("Behandling")))
            .collect();

        all_behandling.sort_by(|a, b| a.0.cmp(b.0));

        for (name, info) in &all_behandling {
            if info.initial_aktivitet.is_some() {
                println!("  [MAIN] {}", name);
            } else {
                println!("  {}", name);
            }
        }

        // 6. Traverse aktivitet flow
        println!("\n\n=== AKTIVITET FLOW ===");

        for (name, info) in &main_behandling_classes {
            if let Some(initial_aktivitet) = &info.initial_aktivitet {
                println!("\nFlow for {}:", name);
                println!("  Starting with: {}", initial_aktivitet);

                let mut visited = std::collections::HashSet::new();
                traverse_aktivitet_flow(initial_aktivitet, &processor_index, &mut visited, 1);

                // Detect and report cycles for this flow
                let cycles = detect_cycles(initial_aktivitet, &processor_index);
                if !cycles.is_empty() {
                    println!("\n  🔄 Detected {} cycle(s) in this flow:", cycles.len());
                    let mut cycle_pairs: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    for (from, to) in &cycles {
                        let pair_desc = format!(
                            "    {} ↩ {}",
                            shorten_aktivitet_name(from),
                            shorten_aktivitet_name(to)
                        );
                        cycle_pairs.insert(pair_desc);
                    }
                    let mut pairs: Vec<_> = cycle_pairs.into_iter().collect();
                    pairs.sort();
                    for pair in pairs {
                        println!("{}", pair);
                    }
                }
            }
        }
    }

    // 7. Generate DOT graph and convert to requested format
    println!("\n📊 Generating graphs...");

    // Determine output directory
    let output_dir = args
        .output_dir
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap());

    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)
            .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;
    }

    let mut generated_files = Vec::new();

    for (name, info) in &main_behandling_classes {
        if let Some(initial_aktivitet) = &info.initial_aktivitet {
            let dot_content = generate_dot_graph(
                name,
                initial_aktivitet,
                &processor_index,
                &class_index,
                &args.edge_style,
                args.show_conditions,
                args.show_legend,
                !args.no_deduplicate,
            )?;

            let dot_filename = output_dir.join(format!("{}_flow.dot", name));
            fs::write(&dot_filename, dot_content)
                .with_context(|| format!("Failed to write DOT file: {:?}", dot_filename))?;

            if args.verbose {
                println!("  ✓ Generated DOT: {}", dot_filename.display());
            }

            // Convert to requested format using graphviz
            let output_filename = output_dir.join(format!("{}_flow.{}", name, args.format));

            let status = Command::new("dot")
                .arg(format!("-T{}", args.format))
                .arg(&dot_filename)
                .arg("-o")
                .arg(&output_filename)
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("  ✅ Generated: {}", output_filename.display());
                    generated_files.push(output_filename.clone());

                    // Delete the .dot file unless --keep-dot is specified
                    if !args.keep_dot {
                        let _ = fs::remove_file(&dot_filename);
                    }
                }
                Ok(s) => {
                    eprintln!(
                        "  ⚠️  Warning: graphviz 'dot' command failed with status: {}",
                        s
                    );
                    eprintln!("     DOT file saved at: {}", dot_filename.display());
                    eprintln!(
                        "     You can manually convert it with: dot -T{} {} -o {}",
                        args.format,
                        dot_filename.display(),
                        output_filename.display()
                    );
                }
                Err(e) => {
                    eprintln!("  ⚠️  Warning: Could not run graphviz 'dot' command: {}", e);
                    eprintln!("     Make sure graphviz is installed (brew install graphviz / apt install graphviz)");
                    eprintln!("     DOT file saved at: {}", dot_filename.display());
                }
            }
        }
    }

    // Open all generated files (if --open is specified)
    if args.open && !generated_files.is_empty() {
        println!("\n🚀 Opening {} file(s)...", generated_files.len());

        for file in &generated_files {
            if args.verbose {
                println!("  Opening {}...", file.display());
            }

            match opener::open(file) {
                Ok(_) => {
                    if args.verbose {
                        println!("    ✓ Opened successfully");
                    }
                }
                Err(e) => {
                    eprintln!(
                        "  ⚠️  Could not automatically open {}: {}",
                        file.display(),
                        e
                    );
                    eprintln!("     Please open manually: {}", file.display());
                }
            }
        }
    }

    println!("\n✨ Done!");
    Ok(())
}

fn traverse_aktivitet_flow(
    aktivitet_name: &str,
    processor_index: &HashMap<String, ProcessorInfo>,
    visited: &mut std::collections::HashSet<String>,
    depth: usize,
) {
    if visited.contains(aktivitet_name) {
        println!(
            "{}  [CYCLE DETECTED: {}]",
            "  ".repeat(depth),
            aktivitet_name
        );
        return;
    }

    visited.insert(aktivitet_name.to_string());

    if let Some(processor) = processor_index.get(aktivitet_name) {
        if processor.next_aktiviteter.is_empty() {
            println!("{}  → [END]", "  ".repeat(depth));
        } else if processor.next_aktiviteter.len() == 1 {
            let next = &processor.next_aktiviteter[0];
            println!("{}  → {}", "  ".repeat(depth), next.aktivitet_name);
            traverse_aktivitet_flow(&next.aktivitet_name, processor_index, visited, depth + 1);
        } else {
            // Multiple branches
            for next in &processor.next_aktiviteter {
                if let Some(condition) = &next.condition {
                    println!(
                        "{}  → [IF {}] {}",
                        "  ".repeat(depth),
                        condition,
                        next.aktivitet_name
                    );
                } else {
                    println!("{}  → [ELSE] {}", "  ".repeat(depth), next.aktivitet_name);
                }
                let mut branch_visited = visited.clone();
                traverse_aktivitet_flow(
                    &next.aktivitet_name,
                    processor_index,
                    &mut branch_visited,
                    depth + 1,
                );
            }
        }
    } else {
        println!("{}  → [PROCESSOR NOT FOUND]", "  ".repeat(depth));
    }
}

fn collect_kotlin_files(root: &str) -> Result<Vec<PathBuf>> {
    let mut kt_files = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(ext) = entry.path().extension() {
            if ext == "kt" {
                kt_files.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(kt_files)
}

fn build_class_index(parser: &mut Parser, files: &[PathBuf]) -> Result<HashMap<String, ClassInfo>> {
    let mut index = HashMap::new();

    for file in files {
        let source_code = fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {}", file.display()))?;

        let tree = parser
            .parse(&source_code, None)
            .context("Failed to parse file")?;

        let root_node = tree.root_node();

        // Extract all class declarations
        extract_classes(&source_code, root_node, file, &mut index);
    }

    // Second pass: extract opprettInitiellAktivitet for Behandling classes
    for file in files {
        let source_code = fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {}", file.display()))?;

        let tree = parser
            .parse(&source_code, None)
            .context("Failed to parse file")?;

        let root_node = tree.root_node();

        extract_initial_aktivitet(&source_code, root_node, &mut index);
    }

    Ok(index)
}

fn extract_classes(
    source: &str,
    node: tree_sitter::Node,
    file: &PathBuf,
    index: &mut HashMap<String, ClassInfo>,
) {
    let mut cursor = node.walk();

    // Recursively traverse the tree
    fn visit_node(
        cursor: &mut tree_sitter::TreeCursor,
        source: &str,
        file: &PathBuf,
        index: &mut HashMap<String, ClassInfo>,
    ) {
        let node = cursor.node();

        if node.kind() == "class_declaration" {
            // Extract class name and supertypes
            if let Some(class_info) = extract_class_info(node, source, file) {
                index.insert(class_info.name.clone(), class_info);
            }
        }

        // Recurse into children
        if cursor.goto_first_child() {
            loop {
                visit_node(cursor, source, file, index);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    visit_node(&mut cursor, source, file, index);
}

fn extract_class_info(
    class_node: tree_sitter::Node,
    source: &str,
    file: &PathBuf,
) -> Option<ClassInfo> {
    let mut class_name = None;
    let mut supertypes = Vec::new();

    let mut cursor = class_node.walk();

    // Look for simple_identifier (class name) and delegation_specifier (supertypes)
    for child in class_node.children(&mut cursor) {
        match child.kind() {
            "simple_identifier" | "type_identifier" => {
                if class_name.is_none() {
                    let name = child.utf8_text(source.as_bytes()).ok()?.to_string();
                    class_name = Some(name);
                }
            }
            "delegation_specifier" => {
                if let Some(supertype) = extract_single_supertype(child, source) {
                    supertypes.push(supertype);
                }
            }
            _ => {}
        }
    }

    class_name.map(|name| ClassInfo {
        name,
        file: file.clone(),
        supertypes,
        initial_aktivitet: None,
    })
}

fn extract_single_supertype(delegation_node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = delegation_node.walk();

    for child in delegation_node.children(&mut cursor) {
        if child.kind() == "user_type"
            || child.kind() == "type_identifier"
            || child.kind() == "constructor_invocation"
        {
            return Some(extract_type_name(child, source));
        }
    }

    None
}

fn extract_type_name(node: tree_sitter::Node, source: &str) -> String {
    match node.kind() {
        "user_type" => {
            // For user_type, concatenate all type_identifier children
            let mut cursor = node.walk();
            let mut parts = Vec::new();

            for child in node.children(&mut cursor) {
                if child.kind() == "type_identifier" || child.kind() == "simple_identifier" {
                    if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        parts.push(text.to_string());
                    }
                }
            }

            if !parts.is_empty() {
                parts.join(".")
            } else {
                node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
            }
        }
        "constructor_invocation" => {
            // For constructor invocations like "Behandling()", extract the type
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "user_type"
                    || child.kind() == "type_identifier"
                    || child.kind() == "simple_identifier"
                {
                    return extract_type_name(child, source);
                }
            }
            node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
        }
        _ => node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
    }
}

fn extract_initial_aktivitet(
    source: &str,
    node: tree_sitter::Node,
    index: &mut HashMap<String, ClassInfo>,
) {
    let mut cursor = node.walk();

    fn visit_node(
        cursor: &mut tree_sitter::TreeCursor,
        source: &str,
        index: &mut HashMap<String, ClassInfo>,
        current_class: &mut Option<String>,
    ) {
        let node = cursor.node();

        match node.kind() {
            "class_declaration" => {
                // Track which class we're in
                let mut class_cursor = node.walk();
                for child in node.children(&mut class_cursor) {
                    if child.kind() == "type_identifier" || child.kind() == "simple_identifier" {
                        if let Ok(name) = child.utf8_text(source.as_bytes()) {
                            *current_class = Some(name.to_string());
                            break;
                        }
                    }
                }
            }
            "function_declaration" => {
                // Check if this is opprettInitiellAktivitet
                if let Some(class_name) = current_class {
                    if is_opprett_initiell_aktivitet(node, source) {
                        if let Some(aktivitet_name) =
                            extract_return_type_from_function(node, source)
                        {
                            if let Some(class_info) = index.get_mut(class_name) {
                                class_info.initial_aktivitet = Some(aktivitet_name);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Recurse into children
        if cursor.goto_first_child() {
            loop {
                visit_node(cursor, source, index, current_class);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    let mut current_class = None;
    visit_node(&mut cursor, source, index, &mut current_class);
}

fn is_opprett_initiell_aktivitet(func_node: tree_sitter::Node, source: &str) -> bool {
    let mut cursor = func_node.walk();
    for child in func_node.children(&mut cursor) {
        if child.kind() == "simple_identifier" {
            if let Ok(name) = child.utf8_text(source.as_bytes()) {
                return name == "opprettInitiellAktivitet";
            }
        }
    }
    false
}

fn extract_return_type_from_function(func_node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = func_node.walk();

    // First, try to find a call_expression in the function body
    for child in func_node.children(&mut cursor) {
        if child.kind() == "function_body" {
            if let Some(call_type) = find_constructor_call(child, source) {
                return Some(call_type);
            }
        }
    }

    None
}

fn build_processor_index(
    parser: &mut Parser,
    files: &[PathBuf],
) -> Result<HashMap<String, ProcessorInfo>> {
    let mut index = HashMap::new();

    for file in files {
        let source_code = fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {}", file.display()))?;

        let tree = parser
            .parse(&source_code, None)
            .context("Failed to parse file")?;

        let root_node = tree.root_node();

        extract_processors(&source_code, root_node, &mut index);
    }

    Ok(index)
}

fn extract_processors(
    source: &str,
    node: tree_sitter::Node,
    index: &mut HashMap<String, ProcessorInfo>,
) {
    let mut cursor = node.walk();

    fn visit_node(
        cursor: &mut tree_sitter::TreeCursor,
        source: &str,
        index: &mut HashMap<String, ProcessorInfo>,
        current_class: &mut Option<String>,
        current_aktivitet_class: &mut Option<String>,
    ) {
        let node = cursor.node();

        match node.kind() {
            "class_declaration" => {
                // Extract class name
                let mut class_cursor = node.walk();
                for child in node.children(&mut class_cursor) {
                    if child.kind() == "type_identifier" || child.kind() == "simple_identifier" {
                        if let Ok(name) = child.utf8_text(source.as_bytes()) {
                            *current_class = Some(name.to_string());

                            // Check if this is a processor (ends with Processor)
                            if name.ends_with("Processor") {
                                // Try to extract the aktivitet class from the supertype
                                if let Some(aktivitet) =
                                    extract_aktivitet_from_processor(node, source)
                                {
                                    *current_aktivitet_class = Some(aktivitet);
                                }
                            }
                            break;
                        }
                    }
                }
            }
            "function_declaration" => {
                // Check if this is doProcess or onFinished
                if let Some(processor_class) = current_class {
                    if let Some(aktivitet_class) = current_aktivitet_class {
                        if is_do_process_function(node, source)
                            || is_on_finished_function(node, source)
                        {
                            let next_aktiviteter = extract_neste_aktivitet_calls(node, source);
                            let has_manuell = has_manuell_behandling_call(node, source);
                            // Always add to index, even with empty next_aktiviteter (end state)
                            // Check if we already have an entry for this aktivitet
                            if let Some(existing) = index.get_mut(aktivitet_class) {
                                // Merge the next aktiviteter
                                for next in next_aktiviteter {
                                    if !existing
                                        .next_aktiviteter
                                        .iter()
                                        .any(|n| n.aktivitet_name == next.aktivitet_name)
                                    {
                                        existing.next_aktiviteter.push(next);
                                    }
                                }
                                // Update manuell flag if found
                                if has_manuell {
                                    existing.has_manuell_behandling = true;
                                }
                            } else {
                                // Create new entry
                                index.insert(
                                    aktivitet_class.clone(),
                                    ProcessorInfo {
                                        processor_class: processor_class.clone(),
                                        next_aktiviteter,
                                        has_manuell_behandling: has_manuell,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Recurse into children
        if cursor.goto_first_child() {
            loop {
                visit_node(
                    cursor,
                    source,
                    index,
                    current_class,
                    current_aktivitet_class,
                );
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    let mut current_class = None;
    let mut current_aktivitet_class = None;
    visit_node(
        &mut cursor,
        source,
        index,
        &mut current_class,
        &mut current_aktivitet_class,
    );
}

fn extract_aktivitet_from_processor(class_node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = class_node.walk();

    for child in class_node.children(&mut cursor) {
        if child.kind() == "delegation_specifier" {
            // Look for the type parameter in the supertype
            if let Some(type_param) = extract_type_parameter(child, source) {
                return Some(type_param);
            }
        }
    }

    None
}

fn extract_type_parameter(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "constructor_invocation" {
            // The type_arguments are inside the user_type
            let mut user_type_cursor = child.walk();
            for ut_child in child.children(&mut user_type_cursor) {
                if ut_child.kind() == "user_type" {
                    // The type_arguments are inside the user_type
                    let mut type_args_cursor = ut_child.walk();
                    for arg in ut_child.children(&mut type_args_cursor) {
                        if arg.kind() == "type_arguments" {
                            let mut args_cursor = arg.walk();
                            let mut type_projections = Vec::new();

                            // Collect all type projections
                            for type_arg in arg.children(&mut args_cursor) {
                                if type_arg.kind() == "type_projection" {
                                    let mut proj_cursor = type_arg.walk();
                                    for type_node in type_arg.children(&mut proj_cursor) {
                                        if type_node.kind() == "user_type"
                                            || type_node.kind() == "type_identifier"
                                        {
                                            type_projections
                                                .push(extract_type_name(type_node, source));
                                        }
                                    }
                                }
                            }

                            // Return the second type parameter (aktivitet class)
                            // For AktivitetProcessor<Behandling, Aktivitet>, we want the second one (index 1)
                            // For AldeAktivitetProcessor<Behandling, Aktivitet, G, V, S, F>, we also want the second one (index 1)
                            if type_projections.len() >= 2 {
                                return Some(type_projections[1].clone());
                            } else if type_projections.len() == 1 {
                                return Some(type_projections[0].clone());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn is_do_process_function(node: tree_sitter::Node, source: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "simple_identifier" {
            if let Ok(name) = child.utf8_text(source.as_bytes()) {
                return name == "doProcess";
            }
        }
    }
    false
}

fn is_on_finished_function(func_node: tree_sitter::Node, source: &str) -> bool {
    let mut cursor = func_node.walk();
    for child in func_node.children(&mut cursor) {
        if child.kind() == "simple_identifier" {
            if let Ok(name) = child.utf8_text(source.as_bytes()) {
                return name == "onFinished";
            }
        }
    }
    false
}

fn has_manuell_behandling_call(func_node: tree_sitter::Node, source: &str) -> bool {
    fn search_node(node: tree_sitter::Node, source: &str) -> bool {
        // Check if this is an assignment with manuellBehandling
        if node.kind() == "assignment" {
            // Check the entire assignment text for the pattern
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                if text.contains("manuellBehandling") && text.contains("ManuellBehandling") {
                    return true;
                }
            }
        }

        // Recursively search children
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if search_node(cursor.node(), source) {
                    return true;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        false
    }

    search_node(func_node, source)
}

fn extract_neste_aktivitet_calls(func_node: tree_sitter::Node, source: &str) -> Vec<NextAktivitet> {
    let mut aktiviteter = Vec::new();
    let mut cursor = func_node.walk();

    // Look for the function body
    for child in func_node.children(&mut cursor) {
        if child.kind() == "function_body" {
            find_neste_aktivitet_in_node(child, source, &mut aktiviteter, None);
        }
    }

    // If no nesteAktivitet calls found, check if it's an end state (aktivitetFullfort)
    // Empty list means end state
    aktiviteter
}

fn find_neste_aktivitet_in_node(
    node: tree_sitter::Node,
    source: &str,
    aktiviteter: &mut Vec<NextAktivitet>,
    condition: Option<String>,
) {
    let mut cursor = node.walk();

    match node.kind() {
        "call_expression" => {
            // Check if this is a nesteAktivitet call
            if is_neste_aktivitet_call(node, source) {
                if let Some(aktivitet_name) = extract_aktivitet_from_call(node, source) {
                    aktiviteter.push(NextAktivitet {
                        aktivitet_name,
                        condition: condition.clone(),
                        is_collection: false,
                    });
                }
            }
            // Check if this is a collection operation that creates multiple aktiviteter
            else if is_collection_operation(node, source) {
                if let Some(aktivitet_name) = extract_aktivitet_from_collection_call(node, source) {
                    aktiviteter.push(NextAktivitet {
                        aktivitet_name,
                        condition: condition.clone(),
                        is_collection: true,
                    });
                }
            }
            // Check if this is a nesteAktiviteter() call with a collection pattern
            else if is_neste_aktiviteter_call(node, source) {
                if let Some(aktivitet_names) =
                    extract_aktiviteter_from_collection_pattern(node, source)
                {
                    for aktivitet_name in aktivitet_names {
                        aktiviteter.push(NextAktivitet {
                            aktivitet_name,
                            condition: condition.clone(),
                            is_collection: true,
                        });
                    }
                }
            }
            // Note: aktivitetFullfort() calls are ignored - they indicate end state
            // which is represented by empty next_aktiviteter list
        }
        "if_expression" => {
            // Extract the condition
            let mut if_cursor = node.walk();
            let mut condition_text = None;

            for child in node.children(&mut if_cursor) {
                if child.kind() == "(" {
                    // Next sibling should be the condition
                    continue;
                } else if condition_text.is_none()
                    && child.kind() != "if"
                    && child.kind() != "control_structure_body"
                {
                    if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        condition_text = Some(text.to_string());
                    }
                }
            }

            // Process if and else branches
            let mut if_cursor = node.walk();
            let mut branch_count = 0;
            for child in node.children(&mut if_cursor) {
                if child.kind() == "control_structure_body" || child.kind() == "call_expression" {
                    branch_count += 1;
                    let branch_condition = if branch_count == 1 {
                        condition_text.clone()
                    } else {
                        condition_text.as_ref().map(|c| format!("NOT ({})", c))
                    };
                    find_neste_aktivitet_in_node(child, source, aktiviteter, branch_condition);
                }
            }
        }
        "return_expression" => {
            // Look for nesteAktivitet in return statement
            if cursor.goto_first_child() {
                loop {
                    find_neste_aktivitet_in_node(
                        cursor.node(),
                        source,
                        aktiviteter,
                        condition.clone(),
                    );
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }
        _ => {
            // For other node types, recursively search children without duplicate processing
        }
    }

    // Recursively search all children, but avoid duplicate processing
    if cursor.goto_first_child() {
        loop {
            find_neste_aktivitet_in_node(cursor.node(), source, aktiviteter, condition.clone());
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

/// Check if a call expression is a collection operation that might create multiple aktiviteter
fn is_collection_operation(node: tree_sitter::Node, source: &str) -> bool {
    let mut cursor = node.walk();

    // Look for patterns like: someCollection.map { ... } or someCollection.forEach { ... }
    for child in node.children(&mut cursor) {
        if child.kind() == "navigation_expression" {
            // Get the full navigation text and check if it ends with collection method
            if let Ok(nav_text) = child.utf8_text(source.as_bytes()) {
                if nav_text.ends_with(".map")
                    || nav_text.ends_with(".forEach")
                    || nav_text.ends_with(".flatMap")
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Extract aktivitet name from a collection operation using pure AST traversal
fn extract_aktivitet_from_collection_call(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "call_suffix" {
            if let Some(name) = extract_from_lambda_in_suffix(child, source) {
                return Some(name);
            }
        }
    }
    None
}

/// Extract activity name from lambda within call suffix using pure AST traversal
fn extract_from_lambda_in_suffix(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut activities = Vec::new();
    extract_activities_from_ast_node(node, source, &mut activities);
    activities.into_iter().next()
}

/// Check if a call expression is a nesteAktiviteter() call
fn is_neste_aktiviteter_call(node: tree_sitter::Node, source: &str) -> bool {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "simple_identifier" {
            if let Ok(name) = child.utf8_text(source.as_bytes()) {
                if name == "nesteAktiviteter" {
                    return true;
                }
            }
        }
    }
    false
}

/// Extract aktivitet names from nesteAktiviteter() call with collection patterns
fn extract_aktiviteter_from_collection_pattern(
    node: tree_sitter::Node,
    source: &str,
) -> Option<Vec<String>> {
    let mut aktivitet_names = Vec::new();
    let mut cursor = node.walk();

    // Walk through all children to find value_arguments
    for child in node.children(&mut cursor) {
        if child.kind() == "call_suffix" {
            extract_from_call_suffix(child, source, &mut aktivitet_names);
        }
    }

    if aktivitet_names.is_empty() {
        None
    } else {
        Some(aktivitet_names)
    }
}

/// Extract from call suffix using pure AST traversal
fn extract_from_call_suffix(
    node: tree_sitter::Node,
    source: &str,
    aktivitet_names: &mut Vec<String>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "value_arguments" {
            extract_from_value_arguments(child, source, aktivitet_names);
        }
    }
}

/// Extract from value arguments using pure AST traversal
fn extract_from_value_arguments(
    node: tree_sitter::Node,
    source: &str,
    aktivitet_names: &mut Vec<String>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_activities_from_ast_node(child, source, aktivitet_names);
    }
}

/// Extract activities from any AST node recursively
fn extract_activities_from_ast_node(
    node: tree_sitter::Node,
    source: &str,
    aktivitet_names: &mut Vec<String>,
) {
    match node.kind() {
        "call_expression" => {
            // Check if this is a direct activity constructor call
            if let Some(activity_name) = extract_constructor_name(node, source) {
                if is_likely_aktivitet_class(&activity_name) {
                    aktivitet_names.push(activity_name);
                }
            } else {
                // Not a constructor, recursively search children
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    extract_activities_from_ast_node(child, source, aktivitet_names);
                }
            }
        }
        "lambda_literal" | "function_literal" => {
            // Search inside lambda expressions for activity constructors
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                extract_activities_from_ast_node(child, source, aktivitet_names);
            }
        }
        _ => {
            // For all other node types, recursively search children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                extract_activities_from_ast_node(child, source, aktivitet_names);
            }
        }
    }
}

/// Extract aktivitet names from binary expressions (like it.map {...} + SomeActivity())
fn extract_aktiviteter_from_binary_expression(
    node: tree_sitter::Node,
    source: &str,
    aktivitet_names: &mut Vec<String>,
) {
    // Use pure AST traversal for binary expressions
    extract_activities_from_ast_node(node, source, aktivitet_names);
}

/// Find nesteAktivitet calls within lambda expressions using pure AST traversal
fn find_nested_aktivitet_in_lambda(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut activities = Vec::new();
    extract_activities_from_ast_node(node, source, &mut activities);
    activities.into_iter().next()
}

/// Search deeply for nesteAktiviteter function calls within complex expressions
fn search_for_nested_neste_aktiviteter(
    node: tree_sitter::Node,
    source: &str,
    aktiviteter: &mut Vec<NextAktivitet>,
    condition: Option<String>,
) {
    let mut cursor = node.walk();

    // Check if this node is itself a nesteAktiviteter call
    if node.kind() == "call_expression" && is_neste_aktiviteter_call(node, source) {
        if let Some(aktivitet_names) = extract_aktiviteter_from_collection_pattern(node, source) {
            for aktivitet_name in aktivitet_names {
                aktiviteter.push(NextAktivitet {
                    aktivitet_name,
                    condition: condition.clone(),
                    is_collection: true,
                });
            }
        }
        return;
    }

    // Recursively search all children
    for child in node.children(&mut cursor) {
        if let Ok(child_text) = child.utf8_text(source.as_bytes()) {
            if child_text.contains("nesteAktiviteter(") {
                search_for_nested_neste_aktiviteter(child, source, aktiviteter, condition.clone());
            }
        }
    }
}

/// Extract aktiviteter from generic nesteAktiviteter patterns like:
/// nesteAktiviteter(it.map { ... } + SomeActivity())
/// nesteAktiviteter(listOf(Activity1(), Activity2()))
fn extract_aktiviteter_from_generic_nesteAktiviteter_pattern(
    node: tree_sitter::Node,
    source: &str,
    aktiviteter: &mut Vec<NextAktivitet>,
    condition: Option<String>,
) {
    if let Ok(text) = node.utf8_text(source.as_bytes()) {
        // Only process if this contains nesteAktiviteter and hasn't been processed by other methods
        if text.contains("nesteAktiviteter(") && !text.contains("nesteAktivitet(") {
            extract_all_activity_constructors(text, aktiviteter, condition);
        }
    }
}

/// Extract all activity constructor calls from nesteAktiviteter text
fn extract_all_activity_constructors(
    text: &str,
    aktiviteter: &mut Vec<NextAktivitet>,
    condition: Option<String>,
) {
    // Find all constructor patterns: ClassName() or ClassName(params)
    let mut pos = 0;
    let mut found_activities = std::collections::HashSet::new();

    while pos < text.len() {
        if let Some(constructor_match) = find_next_constructor(&text[pos..]) {
            let full_pos = pos + constructor_match.start;
            let class_name = constructor_match.name;

            // Check if this looks like an Aktivitet class and we haven't seen it before
            if is_likely_aktivitet_class(&class_name) && !found_activities.contains(&class_name) {
                found_activities.insert(class_name.clone());

                // Determine if this is part of a collection operation (it.map, forEach, etc.)
                let is_collection = text.contains("it.map")
                    || text.contains(".forEach")
                    || text.contains(".flatMap");

                aktiviteter.push(NextAktivitet {
                    aktivitet_name: class_name,
                    condition: condition.clone(),
                    is_collection,
                });
            }

            pos = full_pos + constructor_match.length;
        } else {
            break;
        }
    }
}

struct ConstructorMatch {
    start: usize,
    length: usize,
    name: String,
}

/// Find the next constructor call pattern in the text
fn find_next_constructor(text: &str) -> Option<ConstructorMatch> {
    // Look for pattern: UpperCaseIdentifier(
    let mut pos = 0;
    let chars: Vec<char> = text.chars().collect();

    while pos < chars.len() {
        // Look for uppercase letter (start of class name)
        if chars[pos].is_ascii_uppercase() {
            let start_pos = pos;

            // Collect the class name (alphanumeric + underscore)
            while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                pos += 1;
            }

            // Check if followed by opening parenthesis
            if pos < chars.len() && chars[pos] == '(' {
                let class_name: String = chars[start_pos..pos].iter().collect();
                return Some(ConstructorMatch {
                    start: start_pos,
                    length: pos - start_pos + 1,
                    name: class_name,
                });
            }
        }
        pos += 1;
    }

    None
}

/// Check if the current position is inside a collection operation like it.map
fn is_inside_collection_operation(preceding_text: &str) -> bool {
    // Look for collection operations in the preceding text
    let collection_patterns = ["it.map", ".map", ".forEach", ".flatMap"];

    for pattern in &collection_patterns {
        if let Some(last_occurrence) = preceding_text.rfind(pattern) {
            // Check if there's a closing } after the pattern but before our position
            let after_pattern = &preceding_text[last_occurrence + pattern.len()..];
            if !after_pattern.contains('}') {
                return true;
            }
        }
    }

    false
}

/// Heuristic to determine if a class name looks like an Aktivitet
fn is_likely_aktivitet_class(class_name: &str) -> bool {
    // Must be a valid identifier (alphanumeric + underscore)
    if !class_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return false;
    }

    // Must start with uppercase letter (class naming convention)
    if !class_name
        .chars()
        .next()
        .unwrap_or('a')
        .is_ascii_uppercase()
    {
        return false;
    }

    // Must be reasonable length for a class name
    if class_name.len() < 3 || class_name.len() > 100 {
        return false;
    }

    // Check for aktivitet patterns
    class_name.ends_with("Aktivitet")
        || class_name.ends_with("Activity")
        || class_name.contains("Aktivitet")
}

fn is_neste_aktivitet_call(call_node: tree_sitter::Node, source: &str) -> bool {
    let mut cursor = call_node.walk();

    for child in call_node.children(&mut cursor) {
        if child.kind() == "simple_identifier" {
            if let Ok(name) = child.utf8_text(source.as_bytes()) {
                return name == "nesteAktivitet" || name == "nesteAktiviteter";
            }
        }
    }
    false
}

fn extract_aktivitet_from_call(call_node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = call_node.walk();

    for child in call_node.children(&mut cursor) {
        if child.kind() == "call_suffix" {
            // Look for value_arguments inside call_suffix
            let mut suffix_cursor = child.walk();
            for suffix_child in child.children(&mut suffix_cursor) {
                if suffix_child.kind() == "value_arguments" {
                    let mut args_cursor = suffix_child.walk();
                    for arg in suffix_child.children(&mut args_cursor) {
                        if arg.kind() == "value_argument" {
                            // Check for both positional and named arguments
                            if let Some(name) = extract_aktivitet_from_value_argument(arg, source) {
                                return Some(name);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn extract_aktivitet_from_value_argument(
    arg_node: tree_sitter::Node,
    source: &str,
) -> Option<String> {
    let mut cursor = arg_node.walk();

    for child in arg_node.children(&mut cursor) {
        match child.kind() {
            "call_expression" => {
                // Direct constructor call: nesteAktivitet(ActivityName())
                return extract_constructor_name(child, source);
            }
            "simple_identifier" => {
                // This might be a named parameter like "aktivitet ="
                // Continue to next sibling to find the value
                continue;
            }
            _ => {
                // Recursively check this node for call expressions
                if let Some(name) = find_constructor_in_node(child, source) {
                    return Some(name);
                }
            }
        }
    }

    None
}

fn extract_constructor_name(call_node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = call_node.walk();
    for child in call_node.children(&mut cursor) {
        if child.kind() == "simple_identifier" || child.kind() == "type_identifier" {
            if let Ok(name) = child.utf8_text(source.as_bytes()) {
                // Only return if this looks like a class constructor (starts with uppercase)
                if name.chars().next().unwrap_or('a').is_ascii_uppercase() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

fn find_constructor_in_node(node: tree_sitter::Node, source: &str) -> Option<String> {
    if node.kind() == "call_expression" {
        return extract_constructor_name(node, source);
    }

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if let Some(name) = find_constructor_in_node(cursor.node(), source) {
                return Some(name);
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    None
}

fn detect_cycles(
    start: &str,
    processor_index: &HashMap<String, ProcessorInfo>,
) -> Vec<(String, String)> {
    let mut cycles = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut rec_stack = std::collections::HashSet::new();
    let mut parent_map: HashMap<String, Vec<String>> = HashMap::new();

    fn dfs(
        node: &str,
        processor_index: &HashMap<String, ProcessorInfo>,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
        parent_map: &mut HashMap<String, Vec<String>>,
        cycles: &mut Vec<(String, String)>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(processor) = processor_index.get(node) {
            for next in &processor.next_aktiviteter {
                let next_name = &next.aktivitet_name;

                // Track parent relationships
                parent_map
                    .entry(next_name.clone())
                    .or_insert_with(Vec::new)
                    .push(node.to_string());

                if rec_stack.contains(next_name) {
                    // Back edge found - this is a cycle
                    cycles.push((node.to_string(), next_name.clone()));
                } else if !visited.contains(next_name) {
                    dfs(
                        next_name,
                        processor_index,
                        visited,
                        rec_stack,
                        parent_map,
                        cycles,
                    );
                }
            }
        }

        rec_stack.remove(node);
    }

    dfs(
        start,
        processor_index,
        &mut visited,
        &mut rec_stack,
        &mut parent_map,
        &mut cycles,
    );

    cycles
}

fn group_cycles(cycles: &[(String, String)], edges: &[Edge]) -> Vec<Vec<String>> {
    if cycles.is_empty() {
        return Vec::new();
    }

    // Build adjacency map from edges
    let mut adj_map: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        adj_map
            .entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.to.clone());
    }

    // Find all nodes involved in cycles
    let mut cycle_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (from, to) in cycles {
        cycle_nodes.insert(from.clone());
        cycle_nodes.insert(to.clone());
    }

    // Use DFS to find strongly connected components among cycle nodes
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    for node in &cycle_nodes {
        if !visited.contains(node) {
            let mut component = Vec::new();
            let mut stack = vec![node.clone()];
            let mut local_visited = std::collections::HashSet::new();

            while let Some(current) = stack.pop() {
                if local_visited.contains(&current) {
                    continue;
                }
                local_visited.insert(current.clone());

                if cycle_nodes.contains(&current) {
                    component.push(current.clone());
                    visited.insert(current.clone());

                    // Add neighbors that are in cycle_nodes
                    if let Some(neighbors) = adj_map.get(&current) {
                        for neighbor in neighbors {
                            if cycle_nodes.contains(neighbor) && !local_visited.contains(neighbor) {
                                stack.push(neighbor.clone());
                            }
                        }
                    }

                    // Also check reverse edges (nodes that point to current)
                    for (from, to) in cycles {
                        if to == &current && !local_visited.contains(from) {
                            stack.push(from.clone());
                        }
                    }
                }
            }

            if !component.is_empty() {
                groups.push(component);
            }
        }
    }

    groups
}

fn generate_dot_graph(
    behandling_name: &str,
    initial_aktivitet: &str,
    processor_index: &HashMap<String, ProcessorInfo>,
    class_index: &HashMap<String, ClassInfo>,
    edge_style: &str,
    show_conditions: bool,
    show_legend: bool,
    deduplicate: bool,
) -> Result<String> {
    let mut dot = String::new();
    dot.push_str("digraph BehandlingFlow {\n");
    dot.push_str("  rankdir=TB;\n");

    // Set splines based on edge style preference
    match edge_style {
        "straight" | "polyline" => dot.push_str("  splines=polyline;\n"),
        "ortho" | "orthogonal" => dot.push_str("  splines=ortho;\n"),
        "curved" | "spline" => dot.push_str("  splines=spline;\n"),
        _ => dot.push_str("  splines=polyline;\n"), // default to straight
    }

    dot.push_str("  node [shape=box, style=rounded, fontname=\"Arial\"];\n");
    dot.push_str("  edge [fontname=\"Arial\", fontsize=10];\n\n");

    // Add title
    dot.push_str(&format!(
        "  labelloc=\"t\";\n  label=\"{} Flow\";\n  fontsize=16;\n\n",
        behandling_name
    ));

    // Track all nodes and edges to avoid duplicates
    let mut visited_nodes = std::collections::HashSet::new();
    let mut node_definitions = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();

    // Start node
    dot.push_str(&format!(
        "  start [label=\"START\", shape=circle, style=filled, fillcolor=\"#90EE90\"];\n"
    ));
    dot.push_str(&format!(
        "  start -> \"{}\";\n\n",
        escape_label(initial_aktivitet)
    ));

    // Build graph recursively
    build_dot_nodes(
        initial_aktivitet,
        processor_index,
        class_index,
        &mut visited_nodes,
        &mut node_definitions,
        &mut edges,
        &mut std::collections::HashSet::new(),
    );

    // Detect iteration groups
    let iteration_groups = detect_iteration_groups(processor_index, &edges);

    // Detect cycles
    let cycles = detect_cycles(initial_aktivitet, processor_index);

    // Group cycles into strongly connected components
    let cycle_groups = group_cycles(&cycles, &edges);

    // Create a set of all nodes in cycles for easy lookup
    let mut nodes_in_cycles = std::collections::HashSet::new();
    for group in &cycle_groups {
        for node in group {
            nodes_in_cycles.insert(node.clone());
        }
    }

    // Create a set of cycle edges (back edges)
    let cycle_edges: std::collections::HashSet<(String, String)> = cycles.iter().cloned().collect();

    // Add iteration clusters
    for (idx, iteration_group) in iteration_groups.iter().enumerate() {
        if iteration_group.iterated_nodes.len() > 1 {
            dot.push_str(&format!("  subgraph cluster_iteration_{} {{\n", idx));
            dot.push_str("    style=\"rounded,dashed\";\n");
            dot.push_str("    color=\"#4CAF50\";\n");
            dot.push_str("    penwidth=2.5;\n");
            dot.push_str("    bgcolor=\"#F0FFF0\";\n");
            dot.push_str(&format!(
                "    label=\"Loop (triggered by {})\";\n",
                iteration_group.trigger_node
            ));
            dot.push_str("    fontcolor=\"#2E7D32\";\n");
            dot.push_str("    fontsize=12;\n");

            // Add all nodes in the iteration path to the cluster
            for node in &iteration_group.iterated_nodes {
                // Only add if the node has a definition (avoid duplicates and unknown nodes)
                if node_definitions
                    .iter()
                    .any(|def| def.contains(&format!("\"{}\"", escape_label(node))))
                {
                    dot.push_str(&format!("    \"{}\";\n", escape_label(node)));
                }
            }

            dot.push_str("  }\n\n");
        }
    }

    // Add cycle clusters
    for (idx, cycle_nodes) in cycle_groups.iter().enumerate() {
        if cycle_nodes.len() > 1 {
            dot.push_str(&format!("\n  subgraph cluster_{} {{\n", idx));
            dot.push_str("    style=\"rounded,dashed\";\n");
            dot.push_str("    color=\"#FF6B6B\";\n");
            dot.push_str("    penwidth=2.5;\n");
            dot.push_str("    bgcolor=\"#FFF5F5\";\n");
            dot.push_str("    label=\"🔄 Waiting/Retry Loop\";\n");
            dot.push_str("    fontcolor=\"#FF6B6B\";\n");
            dot.push_str("    fontsize=12;\n");
            dot.push_str("    fontname=\"Arial Bold\";\n");

            // Add nodes in this cycle to the cluster
            for node in cycle_nodes {
                dot.push_str(&format!("    \"{}\";\n", escape_label(node)));
            }

            dot.push_str("  }\n");
        }
    }

    // Add node definitions
    for node_def in node_definitions {
        dot.push_str(&format!("  {};\n", node_def));
    }

    // Consolidate and add edges (if deduplication enabled)
    if deduplicate {
        let consolidated = consolidate_edges(&edges, &cycle_edges, show_conditions);
        for edge in consolidated {
            dot.push_str(&format!("  {};\n", edge));
        }
    } else {
        // Add edges without consolidation
        for edge in &edges {
            let dot_edge = if edge.to.starts_with("unknown_") {
                format!(
                    "\"{}\" -> {} [style=dashed]",
                    escape_label(&edge.from),
                    escape_label(&edge.to)
                )
            } else if cycle_edges.contains(&(edge.from.clone(), edge.to.clone())) {
                format!(
                    "\"{}\" -> \"{}\" [color=\"#FF6B6B\", penwidth=2, style=bold, constraint=false{}]",
                    escape_label(&edge.from),
                    escape_label(&edge.to),
                    if show_conditions && !edge.label.is_empty() {
                        format!(", label=\"{}\"", escape_label(&edge.label))
                    } else {
                        String::new()
                    }
                )
            } else if edge.is_collection {
                format!(
                    "\"{}\" -> \"{}\" [label=\"{}\", color=\"#4CAF50\", penwidth=2, style=bold]",
                    escape_label(&edge.from),
                    escape_label(&edge.to),
                    if show_conditions && !edge.label.is_empty() {
                        format!("{} (multiple)", escape_label(&edge.label))
                    } else {
                        "multiple".to_string()
                    }
                )
            } else if show_conditions && !edge.label.is_empty() {
                format!(
                    "\"{}\" -> \"{}\" [label=\"{}\"]",
                    escape_label(&edge.from),
                    escape_label(&edge.to),
                    escape_label(&edge.label)
                )
            } else {
                format!(
                    "\"{}\" -> \"{}\"",
                    escape_label(&edge.from),
                    escape_label(&edge.to)
                )
            };
            dot.push_str(&format!("  {};\n", dot_edge));
        }
    }

    // Add legend as HTML table (if enabled)
    if show_legend {
        dot.push_str("\n  // Legend\n");
        dot.push_str("  {rank=sink;\n");
        dot.push_str("    Legend [shape=none, margin=0, label=<\n");
        dot.push_str(
            "      <TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\" CELLPADDING=\"4\">\n",
        );
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD COLSPAN=\"2\" BGCOLOR=\"#E8E8E8\"><B>Legend</B></TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#90EE90\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">START</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#9370DB\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">AldeAktivitet</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#FFA500\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">📋 Creates Oppgave</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#87CEEB\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">Regular</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#FFD700\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">Waiting</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#FF6B6B\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">Manual</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#FF4444\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">Abort</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#4CAF50\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">Decision</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#FFB6C1\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">END</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("        <TR>\n");
        dot.push_str("          <TD BGCOLOR=\"#CCCCCC\">  </TD>\n");
        dot.push_str("          <TD ALIGN=\"LEFT\">Unknown</TD>\n");
        dot.push_str("        </TR>\n");
        dot.push_str("      </TABLE>\n");
        dot.push_str("    >];\n");
        dot.push_str("  }\n");
    }

    dot.push_str("}\n");
    Ok(dot)
}

/// Detect iteration groups where one aktivitet creates multiple instances of subsequent aktiviteter
fn detect_iteration_groups(
    processor_index: &HashMap<String, ProcessorInfo>,
    edges: &[Edge],
) -> Vec<IterationGroup> {
    let mut iteration_groups = Vec::new();

    // Find all collection edges (fan-out edges)
    let collection_edges: Vec<&Edge> = edges.iter().filter(|e| e.is_collection).collect();

    for collection_edge in collection_edges {
        let trigger_node = collection_edge.from.clone();
        let first_iterated_node = collection_edge.to.clone();

        // Trace the path from the first iterated node to find all nodes in the iteration
        let mut iterated_nodes = vec![first_iterated_node.clone()];
        let mut current_nodes = vec![first_iterated_node];
        let mut visited = std::collections::HashSet::new();

        // Follow the path until we reach an end or cycle back to a known node
        while !current_nodes.is_empty() {
            let mut next_nodes = Vec::new();

            for current_node in &current_nodes {
                if visited.contains(current_node) {
                    continue;
                }
                visited.insert(current_node.clone());

                if let Some(processor) = processor_index.get(current_node) {
                    for next_aktivitet in &processor.next_aktiviteter {
                        // Only include in iteration if it's a direct single path (not conditional)
                        if processor.next_aktiviteter.len() == 1 {
                            next_nodes.push(next_aktivitet.aktivitet_name.clone());
                            iterated_nodes.push(next_aktivitet.aktivitet_name.clone());
                        }
                    }
                }
            }

            current_nodes = next_nodes;

            // Prevent infinite loops
            if visited.len() > 20 {
                break;
            }
        }

        // Only create a group if we have multiple nodes in the iteration path
        if iterated_nodes.len() > 1 {
            iteration_groups.push(IterationGroup {
                trigger_node,
                iterated_nodes,
            });
        }
    }

    iteration_groups
}

fn build_dot_nodes(
    aktivitet_name: &str,
    processor_index: &HashMap<String, ProcessorInfo>,
    class_index: &HashMap<String, ClassInfo>,
    visited_nodes: &mut std::collections::HashSet<String>,
    node_definitions: &mut Vec<String>,
    edges: &mut Vec<Edge>,
    visiting: &mut std::collections::HashSet<String>,
) {
    if visited_nodes.contains(aktivitet_name) {
        return;
    }

    if visiting.contains(aktivitet_name) {
        // Cycle detected
        return;
    }

    visiting.insert(aktivitet_name.to_string());
    visited_nodes.insert(aktivitet_name.to_string());

    // Shorten the name for display
    let display_name = shorten_aktivitet_name(aktivitet_name);

    // Check if this aktivitet creates a manuell behandling
    let creates_oppgave = processor_index
        .get(aktivitet_name)
        .map(|p| p.has_manuell_behandling)
        .unwrap_or(false);

    // Determine node color based on name patterns and type
    let color = if is_alde_aktivitet(aktivitet_name, class_index) {
        "#9370DB" // Medium purple for AldeAktivitet (important)
    } else if creates_oppgave {
        "#FFA500" // Orange for activities that create manual tasks
    } else if aktivitet_name.contains("Vent") || aktivitet_name.contains("Wait") {
        "#FFD700" // Gold for waiting activities
    } else if aktivitet_name.contains("Manuell") || aktivitet_name.contains("Oppgave") {
        "#FF6B6B" // Red for manual activities
    } else if aktivitet_name.contains("Avbryt") || aktivitet_name.contains("Avslag") {
        "#FF4444" // Dark red for abort/rejection
    } else if aktivitet_name.contains("Iverksett") || aktivitet_name.contains("Vedtak") {
        "#4CAF50" // Green for decision/execution
    } else {
        "#87CEEB" // Sky blue for regular activities
    };

    // Add node definition with oppgave indicator if applicable
    let label = if creates_oppgave {
        format!("📋 {}", display_name)
    } else {
        display_name
    };

    node_definitions.push(format!(
        "\"{}\" [label=\"{}\", style=filled, fillcolor=\"{}\"]",
        escape_label(aktivitet_name),
        escape_label(&label),
        color
    ));

    if let Some(processor) = processor_index.get(aktivitet_name) {
        if processor.next_aktiviteter.is_empty() {
            // End node
            node_definitions.push(
                "end [label=\"END\", shape=circle, style=filled, fillcolor=\"#FFB6C1\"]"
                    .to_string(),
            );
            edges.push(Edge {
                from: aktivitet_name.to_string(),
                to: "end".to_string(),
                label: "".to_string(),
                is_collection: false,
            });
        } else if processor.next_aktiviteter.len() == 1 {
            let next = &processor.next_aktiviteter[0];
            let label = if let Some(condition) = &next.condition {
                format_condition_label(condition)
            } else {
                "".to_string()
            };
            edges.push(Edge {
                from: aktivitet_name.to_string(),
                to: next.aktivitet_name.clone(),
                label,
                is_collection: next.is_collection,
            });
            build_dot_nodes(
                &next.aktivitet_name,
                processor_index,
                class_index,
                visited_nodes,
                node_definitions,
                edges,
                visiting,
            );
        } else {
            // Multiple branches - conditional
            for next in processor.next_aktiviteter.iter() {
                let label = if let Some(condition) = &next.condition {
                    format_condition_label(condition)
                } else {
                    "else".to_string()
                };

                edges.push(Edge {
                    from: aktivitet_name.to_string(),
                    to: next.aktivitet_name.clone(),
                    label,
                    is_collection: next.is_collection,
                });

                build_dot_nodes(
                    &next.aktivitet_name,
                    processor_index,
                    class_index,
                    visited_nodes,
                    node_definitions,
                    edges,
                    visiting,
                );
            }
        }
    } else {
        // No processor found - mark as unknown
        let unknown_id = format!("unknown_{}", aktivitet_name);
        node_definitions.push(format!(
            "{} [label=\"?\", shape=diamond, style=filled, fillcolor=\"#CCCCCC\"]",
            escape_label(&unknown_id)
        ));
        edges.push(Edge {
            from: aktivitet_name.to_string(),
            to: unknown_id,
            label: "".to_string(),
            is_collection: false,
        });
    }

    visiting.remove(aktivitet_name);
}

fn consolidate_edges(
    edges: &[Edge],
    cycle_edges: &std::collections::HashSet<(String, String)>,
    show_conditions: bool,
) -> Vec<String> {
    // Group edges by (from, to) pair
    let mut edge_groups: HashMap<(String, String), Vec<String>> = HashMap::new();
    let mut collection_edges: HashMap<(String, String), bool> = HashMap::new();

    for edge in edges {
        let key = (edge.from.clone(), edge.to.clone());
        edge_groups
            .entry(key.clone())
            .or_insert_with(Vec::new)
            .push(edge.label.clone());

        // Track if any edge in this group is a collection edge
        if edge.is_collection {
            collection_edges.insert(key, true);
        }
    }

    let mut result = Vec::new();

    for ((from, to), labels) in edge_groups.iter() {
        // Filter out empty labels and "else" labels, and get unique ones
        let non_empty_labels: Vec<String> = if show_conditions {
            labels
                .iter()
                .filter(|l| !l.is_empty() && *l != "else")
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect()
        } else {
            Vec::new() // Don't show any conditions
        };

        // Check if this is a cycle edge (back edge)
        let is_cycle_edge = cycle_edges.contains(&(from.clone(), to.clone()));

        // Check if this is a collection edge (fan-out)
        let is_collection_edge = collection_edges
            .get(&(from.clone(), to.clone()))
            .unwrap_or(&false);

        let dot_edge = if !show_conditions || (labels.len() == 1 && labels[0].is_empty()) {
            // Single edge with no label (simple transition or dashed edge)
            if to.starts_with("unknown_") {
                format!(
                    "\"{}\" -> {} [style=dashed]",
                    escape_label(from),
                    escape_label(to)
                )
            } else if is_cycle_edge {
                format!(
                    "\"{}\" -> \"{}\" [color=\"#FF6B6B\", penwidth=2, style=bold, constraint=false]",
                    escape_label(from),
                    escape_label(to)
                )
            } else if *is_collection_edge {
                format!(
                    "\"{}\" -> \"{}\" [label=\"multiple\", color=\"#4CAF50\", penwidth=2, style=bold]",
                    escape_label(from),
                    escape_label(to)
                )
            } else {
                format!("\"{}\" -> \"{}\"", escape_label(from), escape_label(to))
            }
        } else if !show_conditions || non_empty_labels.is_empty() {
            // All labels were empty - simple edge
            if is_cycle_edge {
                format!(
                    "\"{}\" -> \"{}\" [color=\"#FF6B6B\", penwidth=2, style=bold, constraint=false]",
                    escape_label(from),
                    escape_label(to)
                )
            } else if *is_collection_edge {
                format!(
                    "\"{}\" -> \"{}\" [label=\"multiple\", color=\"#4CAF50\", penwidth=2, style=bold]",
                    escape_label(from),
                    escape_label(to)
                )
            } else {
                format!("\"{}\" -> \"{}\"", escape_label(from), escape_label(to))
            }
        } else if non_empty_labels.len() == 1 {
            // Single unique condition
            if is_cycle_edge {
                format!(
                    "\"{}\" -> \"{}\" [label=\"{}\", color=\"#FF6B6B\", penwidth=2, style=bold, constraint=false]",
                    escape_label(from),
                    escape_label(to),
                    escape_label(&non_empty_labels[0])
                )
            } else if *is_collection_edge {
                format!(
                    "\"{}\" -> \"{}\" [label=\"{} (multiple)\", color=\"#4CAF50\", penwidth=2, style=bold]",
                    escape_label(from),
                    escape_label(to),
                    escape_label(&non_empty_labels[0])
                )
            } else {
                format!(
                    "\"{}\" -> \"{}\" [label=\"{}\"]",
                    escape_label(from),
                    escape_label(to),
                    escape_label(&non_empty_labels[0])
                )
            }
        } else if non_empty_labels.len() == 1 {
            // Single unique condition - show it
            if is_cycle_edge {
                format!(
                    "\"{}\" -> \"{}\" [label=\"{}\", color=\"#FF6B6B\", penwidth=2, style=bold, constraint=false]",
                    escape_label(from),
                    escape_label(to),
                    escape_label(&non_empty_labels[0])
                )
            } else {
                format!(
                    "\"{}\" -> \"{}\" [label=\"{}\"]",
                    escape_label(from),
                    escape_label(to),
                    escape_label(&non_empty_labels[0])
                )
            }
        } else {
            // Multiple conditions - just show the first one as example (no "alternative paths" text)
            let sample = &non_empty_labels[0];
            let truncated = if sample.len() > 40 {
                format!("{}...", &sample[..40])
            } else {
                sample.clone()
            };
            if is_cycle_edge {
                format!(
                    "\"{}\" -> \"{}\" [label=\"{}\", color=\"#FF6B6B\", penwidth=2, style=bold, constraint=false]",
                    escape_label(from),
                    escape_label(to),
                    escape_label(&truncated)
                )
            } else {
                format!(
                    "\"{}\" -> \"{}\" [label=\"{}\"]",
                    escape_label(from),
                    escape_label(to),
                    escape_label(&truncated)
                )
            }
        };

        result.push(dot_edge);
    }

    result
}

fn is_alde_aktivitet(aktivitet_name: &str, class_index: &HashMap<String, ClassInfo>) -> bool {
    // Check if this class extends AldeAktivitet
    if let Some(class_info) = class_index.get(aktivitet_name) {
        class_info
            .supertypes
            .iter()
            .any(|supertype| supertype.contains("AldeAktivitet"))
    } else {
        false
    }
}

fn shorten_aktivitet_name(name: &str) -> String {
    // Remove common prefixes
    let shortened = name.replace("FleksibelApSak", "").replace("Aktivitet", "");

    // Extract the step number and description
    if let Some(pos) = shortened.find(char::is_alphabetic) {
        if pos > 0 {
            let (num, rest) = shortened.split_at(pos);
            // Add space between number and text for readability
            return format!("{}\n{}", num, rest);
        }
    }

    shortened
}

fn format_condition_label(condition: &str) -> String {
    let mut formatted = condition.to_string();

    // Detect feature toggle patterns
    if formatted.contains("unleashNextService.isEnabled") || formatted.contains("unleashNext") {
        // Extract feature name - look for the first parameter which is the feature flag
        if let Some(start) = formatted.find("isEnabled(") {
            let after_enabled = &formatted[start + 10..];

            // Find the feature flag name (first parameter)
            let feature_part = if let Some(comma_pos) = after_enabled.find(',') {
                &after_enabled[..comma_pos]
            } else if let Some(paren_pos) = after_enabled.find(')') {
                &after_enabled[..paren_pos]
            } else {
                after_enabled
            };

            // Clean up the feature name
            let feature_name = feature_part
                .trim()
                .replace("PenFeature.", "")
                .replace("\"", "");

            // Check if there are additional conditions after the isEnabled call
            let rest_of_condition = if let Some(close_paren) = after_enabled.find(')') {
                let after_close = &after_enabled[close_paren + 1..].trim();
                if after_close.starts_with("&&") {
                    let extra = after_close[2..]
                        .trim()
                        .replace("behandling.", "")
                        .replace("krav.", "");
                    if !extra.is_empty() {
                        format!(" && {}", extra)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            return format!("🚩 FEATURE: {}{}", feature_name.trim(), rest_of_condition);
        }
        // Fallback if we can't extract the name
        formatted = format!("🚩 FEATURE TOGGLE: {}", formatted);
    }

    // Simplify common patterns
    formatted = formatted.replace("behandling.", "");
    formatted = formatted.replace("krav.", "");

    // Truncate very long conditions
    if formatted.len() > 80 {
        format!("{}...", &formatted[..77])
    } else {
        formatted
    }
}

fn escape_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn find_constructor_call(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();

    if node.kind() == "call_expression" {
        // This is a constructor call
        for child in node.children(&mut cursor) {
            if child.kind() == "simple_identifier" || child.kind() == "type_identifier" {
                if let Ok(name) = child.utf8_text(source.as_bytes()) {
                    return Some(name.to_string());
                }
            }
        }
    }

    // Recurse into children
    if cursor.goto_first_child() {
        loop {
            if let Some(result) = find_constructor_call(cursor.node(), source) {
                return Some(result);
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }

    None
}
