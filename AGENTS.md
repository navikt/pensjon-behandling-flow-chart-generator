# AGENTS.md - Guide for AI Agents

This document is specifically written for AI agents (LLMs, coding assistants, etc.) that need to understand, modify, or extend this codebase.

## Table of Contents

- [Project Overview](#project-overview)
- [Architecture at a Glance](#architecture-at-a-glance)
- [Core Data Structures](#core-data-structures)
- [Key Functions](#key-functions)
- [Code Patterns to Follow](#code-patterns-to-follow)
- [CLI Flags](#cli-flags)
- [Kotlin Patterns Recognized](#kotlin-patterns-recognized)
- [Graphviz DOT Generation](#graphviz-dot-generation)
- [Common Modifications](#common-modifications)
- [Testing Strategy](#testing-strategy)
- [Performance Considerations](#performance-considerations)
- [Common Pitfalls](#common-pitfalls)
- [Debugging Tips](#debugging-tips)
- [Dependencies](#dependencies)
- [File Structure](#file-structure)
- [Edge Cases to Handle](#edge-cases-to-handle)
- [Future Enhancement Ideas](#future-enhancement-ideas)
- [Related Resources](#related-resources)

## Project Overview

**Project**: Behandling Flow Analyzer  
**Language**: Rust  
**Purpose**: Analyze Kotlin codebases to extract and visualize treatment (Behandling) flow diagrams  
**Output**: DOT/Graphviz diagrams with automatic rendering to SVG/PNG/PDF

## Architecture at a Glance

```
┌─────────────────────────────────────────────────────────────┐
│ CLI (clap)                                                  │
│ - Arguments: path, format, edge-style, show-conditions     │
└───────────────────┬─────────────────────────────────────────┘
                    │
┌───────────────────▼─────────────────────────────────────────┐
│ File Scanner (walkdir)                                      │
│ - Recursively finds all .kt files                          │
└───────────────────┬─────────────────────────────────────────┘
                    │
┌───────────────────▼─────────────────────────────────────────┐
│ Tree-sitter Parser                                          │
│ - Parses Kotlin AST                                         │
│ - Extracts class info and processor logic                  │
└───────┬─────────────────────┬───────────────────────────────┘
        │                     │
        ▼                     ▼
┌───────────────┐    ┌────────────────────┐
│ Class Index   │    │ Processor Index    │
│ - ClassInfo   │    │ - ProcessorInfo    │
│ - Supertypes  │    │ - NextAktivitet    │
│ - Initial     │    │ - Conditions       │
└───────┬───────┘    └────────┬───────────┘
        │                     │
        └──────────┬──────────┘
                   ▼
        ┌──────────────────────┐
        │ Flow Graph Generator │
        │ - Cycle detection    │
        │ - Edge consolidation │
        │ - DOT generation     │
        └──────────┬───────────┘
                   ▼
        ┌──────────────────────┐
        │ Graphviz Converter   │
        │ - Renders to format  │
        └──────────┬───────────┘
                   ▼
        ┌──────────────────────┐
        │ File Opener (opener) │
        │ - Opens in browser   │
        └──────────────────────┘
```

## Core Data Structures

### ClassInfo
```rust
struct ClassInfo {
    name: String,                    // Class name (e.g., "FleksibelApSakBehandling")
    file: PathBuf,                   // Source file path
    supertypes: Vec<String>,         // Parent classes/interfaces
    initial_aktivitet: Option<String>, // From opprettInitiellAktivitet()
}
```

**Purpose**: Stores metadata about each Kotlin class found in the codebase.

**Key fields**:
- `initial_aktivitet` - Only set for Behandling classes with `opprettInitiellAktivitet()` method

### ProcessorInfo
```rust
struct ProcessorInfo {
    aktivitet_class: String,         // The aktivitet being processed
    processor_class: String,         // The processor class name
    next_aktiviteter: Vec<NextAktivitet>, // Possible next steps
    has_manuell_behandling: bool,    // True if creates manual task/oppgave
}
```

**Purpose**: Represents the flow logic extracted from processor classes.

### NextAktivitet
```rust
struct NextAktivitet {
    aktivitet_name: String,          // Name of next aktivitet
    condition: Option<String>,       // Condition for this path (if any)
}
```

**Purpose**: Represents a single edge in the flow graph with optional condition.

### Edge
```rust
struct Edge {
    from: String,                    // Source node
    to: String,                      // Target node
    label: String,                   // Edge label (condition)
}
```

**Purpose**: Internal representation of graph edges before DOT generation.

## Key Functions

### main() - Entry Point
- Parses CLI arguments
- Orchestrates the entire flow
- Handles output and file opening

### collect_kotlin_files() - File Discovery
- Uses `walkdir` to recursively find `.kt` files
- Returns `Vec<PathBuf>`

### build_class_index() - Class Extraction
- Parses each Kotlin file with tree-sitter
- Extracts class declarations, supertypes, and initial aktivitet
- Returns `HashMap<String, ClassInfo>`

### build_processor_index() - Processor Extraction
- Identifies processor classes (end with "Processor")
- Extracts generic type parameter (aktivitet class)
- Parses `doProcess()` and `onFinished()` methods
- Extracts `nesteAktivitet()` calls with conditions
- Returns `HashMap<String, ProcessorInfo>`

### detect_cycles() - Cycle Detection
- Uses DFS with recursion stack to detect back edges
- Returns `Vec<(String, String)>` of cycle edges

### group_cycles() - Cycle Grouping
- Groups nodes into strongly connected components
- Uses connected component analysis
- Returns `Vec<Vec<String>>` where each inner vec is a cycle group

### generate_dot_graph() - DOT Generation
- Builds DOT graph string
- Applies node colors based on aktivitet type
- Creates cycle clusters (subgraphs)
- Consolidates edges
- Returns `Result<String>` with DOT content

### consolidate_edges() - Edge Deduplication
- Groups edges by (from, to) pair
- Filters "else" labels when conditions hidden
- Handles cycle edges (back edges) specially
- Returns `Vec<String>` of DOT edge statements

### has_manuell_behandling_call() - Manual Task Detection
- Searches function body for `manuellBehandling = ManuellBehandling(...)` pattern
- Returns `bool` indicating if manual task is created
- Used to mark nodes with 📋 emoji and orange color

## Code Patterns to Follow

### 1. Error Handling
Use `anyhow` for error propagation:
```rust
fn my_function() -> Result<T> {
    something.context("Failed to do something")?;
    Ok(result)
}
```

### 2. Tree-sitter Traversal
Always use cursors and check node kinds:
```rust
let mut cursor = node.walk();
for child in node.children(&mut cursor) {
    if child.kind() == "class_declaration" {
        // Process class
    }
}
```

### 3. Condition Extraction
Extract condition text from AST nodes:
```rust
if let Ok(text) = node.utf8_text(source.as_bytes()) {
    // Use text
}
```

### 4. Recursion with Visited Set
Prevent infinite loops:
```rust
fn traverse(node: &str, visited: &mut HashSet<String>) {
    if visited.contains(node) {
        return;
    }
    visited.insert(node.to_string());
    // Process node
}
```

## CLI Flags

| Flag | Short | Default | Purpose |
|------|-------|---------|---------|
| `[PATH]` | - | `.` | Project directory |
| `--format` | `-f` | `svg` | Output format (svg, png, pdf) |
| `--edge-style` | `-e` | `straight` | Edge style (straight, curved, ortho) |
| `--show-conditions` | `-c` | `false` | Show condition labels |
| `--open` | - | `false` | Auto-open generated file |
| `--keep-dot` | `-k` | `false` | Keep intermediate DOT file |
| `--output-dir` | `-o` | `.` | Output directory |
| `--verbose` | `-v` | `false` | Verbose output |

## Kotlin Patterns Recognized

### 1. Behandling Class
```kotlin
class MyBehandling : Behandling() {
    fun opprettInitiellAktivitet(): StartAktivitet {
        return StartAktivitet()
    }
}
```

**Detection**: 
- Class extends something with "Behandling" in name
- Has `opprettInitiellAktivitet()` method
- Method returns an aktivitet class

### 2. Processor Class
```kotlin
class MyAktivitetProcessor : AktivitetProcessor<MyAktivitet>() {
    fun doProcess(aktivitet: MyAktivitet): AktivitetResponse {
        return nesteAktivitet(NextAktivitet())
    }
}
```

**Detection**:
- Class name ends with "Processor"
- Extends `AktivitetProcessor<T>`
- Has `doProcess()` or `onFinished()` method
- Calls `nesteAktivitet()` or `aktivitetFullfort()`

### 3. Conditional Flow
```kotlin
fun doProcess(aktivitet: A): AktivitetResponse {
    return if (condition) {
        nesteAktivitet(B())
    } else {
        nesteAktivitet(C())
    }
}
```

**Extraction**:
- Parses if/when expressions
- Extracts condition text from AST
- Creates multiple NextAktivitet entries

### 4. Feature Toggles
```kotlin
if (unleashNextService.isEnabled("FEATURE_NAME")) {
    nesteAktivitet(A())
}
```

**Detection**:
- Looks for `unleashNextService.isEnabled()`
- Marks condition with 🚩 emoji
- Extracts feature flag name

### 5. End State
```kotlin
fun doProcess(aktivitet: A): AktivitetResponse {
    aktivitetFullfort()
}
```

**Detection**:
- Finds `aktivitetFullfort()` calls
- Adds edge to END node in graph

### 6. Manual Task Creation
```kotlin
fun doProcess(aktivitet: A): AktivitetResponse {
    manuellBehandling = ManuellBehandling(
        kategori = "MANUAL_REVIEW",
        beskrivelse = "Manual review required"
    )
    nesteAktivitet(NextAktivitet())
}
```

**Detection**:
- Looks for `manuellBehandling = ManuellBehandling(...)` pattern
- Marks node with 📋 emoji and orange color
- Indicates where manual intervention is triggered

## Graphviz DOT Generation

### Node Attributes
```dot
"NodeName" [label="Display", style=filled, fillcolor="#COLOR"]
```

**Color scheme**:
- `#90EE90` - START (green)
- `#9370DB` - AldeAktivitet (purple)
- `#FFA500` - Creates manual task/oppgave (orange) - marked with 📋
- `#FFD700` - Waiting activities (gold)
- `#FF6B6B` - Manual intervention (red)
- `#FF4444` - Abort/rejection (dark red)
- `#4CAF50` - Decision/execution (green)
- `#87CEEB` - Regular activities (sky blue)
- `#FFB6C1` - END (pink)
- `#CCCCCC` - Unknown (gray)

### Edge Attributes
```dot
"From" -> "To" [label="condition", color="#COLOR", penwidth=N, style=STYLE]
```

**Styles**:
- Default: No special attributes
- Cycle edge: `color="#FF6B6B", penwidth=2, style=bold, constraint=false`
- Unknown: `style=dashed`

### Cluster (Cycle) Attributes
```dot
subgraph cluster_0 {
    style="rounded,dashed";
    color="#FF6B6B";
    penwidth=2.5;
    bgcolor="#FFF5F5";
    label="🔄 Waiting/Retry Loop";
    "Node1";
    "Node2";
}
```

## Common Modifications

### Adding a New CLI Flag

1. Add to `Args` struct:
```rust
#[derive(ClapParser, Debug)]
struct Args {
    /// New flag description
    #[arg(long)]
    new_flag: bool,
}
```

2. Use in main():
```rust
if args.new_flag {
    // Do something
}
```

3. Update documentation in README.md, QUICKSTART.md, BINARY_USAGE.md

### Adding a New Node Color

1. Update `build_dot_nodes()`:
```rust
let color = if aktivitet_name.contains("MyPattern") {
    "#HEXCOLOR" // Your new color
} else if /* existing conditions */ {
    // ...
}
```

2. Add emoji indicator if needed:
```rust
let label = if some_condition {
    format!("📋 {}", display_name)
} else {
    display_name
};
```

3. Update documentation in README.md (Color-Coded Nodes section)

### Adding a New Edge Style

1. Update `generate_dot_graph()`:
```rust
match edge_style {
    "mynewstyle" => dot.push_str("  splines=mynewstyle;\n"),
    // existing cases
}
```

2. Update CLI help text and documentation

### Modifying Condition Extraction

1. Update `find_neste_aktivitet_in_node()`:
```rust
match node.kind() {
    "my_new_pattern" => {
        // Extract condition
        let condition = /* extract from AST */;
        aktiviteter.push(NextAktivitet {
            aktivitet_name: /* ... */,
            condition: Some(condition),
        });
    }
    // existing cases
}
```

### Adding Support for New Kotlin Patterns

1. Study the AST with tree-sitter playground
2. Update appropriate extraction function
3. Add test case in testdata/
4. Verify with `--verbose` output

## Testing Strategy

### Manual Testing
```bash
# Basic test
cargo run -- testdata/fleksibel_alderspensjon_sak_behandling --verbose

# Cycle test
cargo run -- testdata/cycle_test --open

# All options
cargo run -- testdata/cycle_test \
  --format png \
  --edge-style straight \
  --show-conditions \
  --keep-dot \
  --output-dir test-output \
  --verbose
```

### Test Data Locations
- `testdata/fleksibel_alderspensjon_sak_behandling/` - Real-world example
- `testdata/cycle_test/` - Cycle detection tests

### Adding Test Data
1. Create new directory in `testdata/`
2. Add Kotlin files with Behandling/Aktivitet/Processor classes
3. Test with: `cargo run -- testdata/your_new_test`

## Performance Considerations

1. **File scanning**: Uses `walkdir` with default settings
   - Scales well to 1000+ files
   - No parallelization (not needed for typical use)

2. **Parsing**: Tree-sitter is fast
   - ~1ms per file for typical Kotlin files
   - Can handle files up to 10,000 lines easily

3. **Graph generation**: Linear in nodes and edges
   - O(N) for N aktiviteter
   - Cycle detection is O(N + E)

4. **Graphviz rendering**: External process
   - Can be slow for very large graphs (100+ nodes)
   - SVG rendering is fastest
   - PNG/PDF require more processing

## Common Pitfalls

### 1. Tree-sitter Node Traversal
❌ **Wrong**: Direct child access without cursor
```rust
for child in node.children() { // Missing cursor
    // ...
}
```

✅ **Correct**: Use cursor
```rust
let mut cursor = node.walk();
for child in node.children(&mut cursor) {
    // ...
}
```

### 2. String Escaping in DOT
❌ **Wrong**: Raw strings in DOT
```rust
format!("\"{}\" -> \"{}\"", from, to)
```

✅ **Correct**: Escape labels
```rust
format!("\"{}\" -> \"{}\"", escape_label(from), escape_label(to))
```

### 3. Cycle Detection
❌ **Wrong**: Only checking visited set
```rust
if visited.contains(node) {
    return; // Might not detect cycles
}
```

✅ **Correct**: Use recursion stack
```rust
if rec_stack.contains(node) {
    // This is a back edge - cycle detected!
}
```

### 4. Edge Consolidation
❌ **Wrong**: Creating duplicate edges
```rust
for edge in edges {
    add_to_dot(edge);
}
```

✅ **Correct**: Consolidate first
```rust
let consolidated = consolidate_edges(&edges);
for edge in consolidated {
    add_to_dot(edge);
}
```

## Debugging Tips

### 1. Use --verbose flag
Shows all intermediate steps:
- Files scanned
- Classes found
- Processors detected
- Flow traversal
- Cycles detected

### 2. Use --keep-dot flag
Inspect the generated DOT file:
```bash
cargo run -- path --keep-dot --output-dir debug
cat debug/Flow_flow.dot
```

### 3. Add println! debugging
In appropriate places:
```rust
println!("DEBUG: Processing node: {:?}", node);
println!("DEBUG: Found condition: {:?}", condition);
```

### 4. Tree-sitter debugging
Print AST structure:
```rust
fn print_ast(node: tree_sitter::Node, source: &str, depth: usize) {
    println!("{}{}: {:?}", 
        "  ".repeat(depth),
        node.kind(),
        node.utf8_text(source.as_bytes())
    );
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_ast(child, source, depth + 1);
    }
}
```

## Dependencies

### Runtime
- `tree-sitter` (0.22) - Parser framework
- `tree-sitter-kotlin` (0.3) - Kotlin grammar
- `walkdir` (2.4) - Directory traversal
- `anyhow` (1.0) - Error handling
- `clap` (4.5) - CLI argument parsing
- `opener` (0.7) - Cross-platform file opening

### Build-time
- Rust 1.70+ (uses 2021 edition)
- Cargo

### External (user must install)
- Graphviz (`dot` command) - For rendering graphs

## File Structure

```
behandling-flow/
├── src/
│   └── main.rs              # All code (single file)
├── testdata/                # Test Kotlin files
│   ├── fleksibel.../
│   └── cycle_test/
├── Cargo.toml               # Rust dependencies
├── Cargo.lock               # Locked versions
├── README.md                # User documentation
├── USAGE.md                 # Detailed usage guide
├── QUICKSTART.md            # Quick reference
├── INSTALL.md               # Installation guide
├── BINARY_USAGE.md          # CLI reference
├── CYCLES.md                # Cycle detection doc
├── AGENTS.md                # This file
└── install.sh               # Installation script
```

## Edge Cases to Handle

1. **Missing processors**: Aktivitet without processor → Show "?" node
2. **Circular dependencies**: Handled by cycle detection
3. **Self-loops**: Node pointing to itself → Treated as cycle
4. **Multiple initial aktiviteter**: Tool processes each separately
5. **No Behandling classes found**: Error message, exit gracefully
6. **Empty directories**: Error message about no .kt files
7. **Malformed Kotlin**: Tree-sitter continues, may miss some info
8. **Very large graphs**: Graphviz may be slow, but tool handles it
9. **ManuellBehandling variations**: Different assignment patterns → Tool looks for both keywords in assignment text

## Future Enhancement Ideas

1. **Interactive HTML output**: Replace static images with interactive SVG
2. **Flow statistics**: Count nodes, edges, cycles, complexity metrics
3. **Path analysis**: Find paths between two nodes
4. **Diff mode**: Compare two versions of the same flow
5. **Export to other formats**: JSON, YAML, Mermaid
6. **Configuration file**: `.behandling-flow.toml` for project settings
7. **Watch mode**: Regenerate on file changes
8. **Multiple flows in one diagram**: Show all flows together
9. **Zoom/filter**: Focus on specific subgraphs
10. **Documentation generation**: Auto-generate flow docs from code comments
11. **Extract oppgave details**: Show kategori, beskrivelse in node tooltip or separate legend
12. **Timeline view**: Show temporal aspects (aktivTil dates) of manual tasks

## Related Resources

- Tree-sitter Kotlin grammar: https://github.com/fwcd/tree-sitter-kotlin
- Graphviz documentation: https://graphviz.org/documentation/
- Clap CLI framework: https://docs.rs/clap/
- Rust book: https://doc.rust-lang.org/book/

## Questions?

If you're an AI agent and something is unclear:
1. Check the relevant section in this document
2. Read the actual code in `src/main.rs` (it's well-commented)
3. Look at test data in `testdata/` for examples
4. Run with `--verbose` to see what's happening
5. Check other documentation files (README.md, USAGE.md, etc.)

Good luck! You've got this! 🤖✨