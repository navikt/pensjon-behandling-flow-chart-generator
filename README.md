# Behandling Flow Analyzer

A Rust CLI tool for analyzing Kotlin codebases to extract and visualize treatment (Behandling) flows.

## Overview

This tool uses Tree-sitter to parse Kotlin files and extract information about classes, their inheritance relationships, and treatment flow patterns. It automatically generates and opens beautiful flow diagrams showing the complete flow through your Behandling system.

## Installation

### Prerequisites

- **Rust** (1.70 or later) - [Install from rustup.rs](https://rustup.rs/)
- **Graphviz** (for visualization) - Required to generate diagrams

Install Graphviz:
```bash
# macOS
brew install graphviz

# Ubuntu/Debian
sudo apt-get install graphviz

# Windows
choco install graphviz
```

### Building the Binary

```bash
cd behandling-flow
cargo build --release
```

The binary will be created at `target/release/behandling-flow`.

### Add to PATH (Optional)

To use the tool from anywhere:

```bash
# Copy to a directory in your PATH
sudo cp target/release/behandling-flow /usr/local/bin/

# Or add an alias to your shell profile (~/.bashrc, ~/.zshrc, etc.)
alias behandling-flow='/path/to/behandling-flow/target/release/behandling-flow'
```

## Usage

### Basic Usage

```bash
# Analyze current directory
behandling-flow

# Analyze a specific directory
behandling-flow /path/to/kotlin/project

# Analyze with verbose output
behandling-flow /path/to/project --verbose
```

### Command-Line Options

```
behandling-flow [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to the Kotlin project directory (defaults to current directory)

Options:
  -f, --format <FORMAT>          Output format: svg, png, pdf, etc. [default: svg]
  -e, --edge-style <EDGE_STYLE>  Edge style: curved, straight, or ortho [default: straight]
  -c, --show-conditions          Show condition labels on edges (default: hidden)
  -l, --show-legend              Show color legend in graph (default: hidden)
      --open                     Automatically open the generated graph
  -k, --keep-dot                 Keep the intermediate .dot file
  -o, --output-dir <OUTPUT_DIR>  Output directory for generated files
  -v, --verbose                  Verbose output
  -h, --help                     Print help
  -V, --version                  Print version
```

### Examples

```bash
# Generate PNG instead of SVG
behandling-flow /path/to/project --format png

# Use curved edges instead of straight
behandling-flow /path/to/project --edge-style curved

# Use orthogonal (right-angle) edges
behandling-flow /path/to/project --edge-style ortho

# Show condition labels on edges
behandling-flow /path/to/project --show-conditions

# Show color legend in graph
behandling-flow /path/to/project --show-legend

# Generate and open automatically
behandling-flow /path/to/project --open

# Save output to specific directory
behandling-flow /path/to/project --output-dir ./graphs

# Keep the intermediate .dot file for manual editing
behandling-flow /path/to/project --keep-dot

# Show detailed analysis output
behandling-flow /path/to/project --verbose

# Combine options
behandling-flow /path/to/project --format pdf --edge-style straight --show-conditions --keep-dot --output-dir ./output --verbose
```

## What It Does

1. **Scans** all `.kt` files in the specified directory
2. **Parses** Kotlin code using Tree-sitter to build an AST
3. **Extracts** Behandling classes and their aktivitet flows
4. **Analyzes** processor logic to trace the complete flow
5. **Generates** a DOT graph file
6. **Converts** the DOT file to your chosen format (SVG, PNG, PDF, etc.)
7. **Opens** the generated visualization automatically (if `--open` flag is used)

## Example Output

```
🔍 Scanning directory: /path/to/kotlin/project
📄 Scanned 54 .kt files
📚 Indexed 122 classes
⚙️  Found 27 processors

📊 Generating graphs...
  ✅ Generated: FleksibelApSakBehandling_flow.svg

✨ Done!
```

With `--open` flag:
```
...
📊 Generating graphs...
  ✅ Generated: FleksibelApSakBehandling_flow.svg

🚀 Opening FleksibelApSakBehandling_flow.svg...

✨ Done!
```

## Diagram Features

The generated flow diagram includes:

### Color-Coded Nodes

- 🟢 **Green (START)** - Entry point
- 🟣 **Purple** - AldeAktivitet (important activities with grunnlag/vurdering pattern)
- 🟠 **Orange (📋)** - Creates manual task (manuellBehandling)
- 🔵 **Sky Blue** - Regular processing activities
- 🟡 **Gold** - Waiting/pause activities (Vent)
- 🔴 **Red** - Manual intervention required (Manuell, Oppgave)
- 🟥 **Dark Red** - Abort/rejection activities (Avbryt, Avslag)
- 🟩 **Green** - Decision/execution activities (Vedtak, Iverksett)
- 🩷 **Pink (END)** - Terminal nodes
- ⚪ **Gray (?)** - Unknown/missing processors

### Smart Features

- **Optional condition labels** - Use `--show-conditions` to display edge labels with actual Kotlin conditions
  - Default: Clean graphs without labels for better visual overview
  - With flag: Shows conditions like `harData`, `NOT (isValid())`, etc.
  - Feature toggles marked with 🚩 emoji when shown (e.g., `🚩 FEATURE: PEN_VURDER_SAMBOER`)
- **Shortened names** for readability (removes common prefixes)
- **Dashed lines** for incomplete/missing processor connections
- **No clutter** - Removed "else" and "alternative paths" labels for cleaner graphs
- **Cycle detection** - Automatically detects and visually highlights cycles/loops in the flow
  - Cycles are enclosed in a red dashed box labeled "🔄 Waiting/Retry Loop"
  - Back edges (edges that create the cycle) are shown in red with bold styling
  - Multiple separate cycles are each grouped in their own cluster
  - Perfect for identifying waiting states and retry logic
- **Configurable edge styles**
  - `straight` (default) - Straight line segments with right angles
  - `curved` - Smooth bezier curves for a flowing appearance
  - `ortho` - Strictly orthogonal (horizontal/vertical only) edges
- **Clean by default** - Condition labels hidden for better visual clarity (use `--show-conditions` to enable)
- **Optional legend** - Add `--show-legend` to include a compact color legend explaining node types

## Current Features

### Completed
- ✅ CLI with proper argument parsing and help
- ✅ Defaults to current directory if no path specified
- ✅ Automatic graph generation with Graphviz
- ✅ Automatic opening of generated files (with --open flag)
- ✅ Multiple output formats (SVG, PNG, PDF, etc.)
- ✅ Configurable edge styles (straight, curved, ortho)
- ✅ Optional condition labels (clean graphs by default)
- ✅ Extract `opprettInitiellAktivitet` function references
- ✅ Identify all `Aktivitet` subclasses
- ✅ Scan processor classes and extract flow logic
- ✅ Build treatment flow graph with conditional branches
- ✅ Generate DOT/Graphviz output
- ✅ Show all conditional paths with actual code conditions
- ✅ Detect and highlight feature toggles (unleashNextService)
- ✅ Detect `aktivitetFullfort()` as end state (activities flow to END node)
- ✅ Support both `doProcess()` and `onFinished()` method patterns
- ✅ Special purple highlighting for AldeAktivitet classes
- ✅ Automatic edge consolidation (groups multiple conditions between same nodes)
- ✅ Cycle detection with visual grouping (waiting/retry loops highlighted)
- ✅ Manual task detection (activities that create manuellBehandling marked with 📋)
- ✅ Optional legend showing all node colors and their meanings (with `--show-legend`)

### Planned
- [ ] Add metadata (step numbers, descriptions from annotations)
- [ ] Interactive HTML visualization
- [ ] Flow statistics and metrics
- [ ] Export to multiple formats simultaneously
- [ ] Configuration file support

## Architecture

### Main Components

1. **CLI Argument Parser** (`clap`) - Handles command-line input with proper help and validation
2. **File Walker** (`walkdir`) - Recursively collects `.kt` files
3. **Tree-sitter Parser** - Parses Kotlin source code into AST
4. **Class Extractor** - Walks AST to extract class information:
   - Finds `class_declaration` nodes
   - Extracts class names from `type_identifier` nodes
   - Extracts supertypes from `delegation_specifier` nodes
   - Detects `opprettInitiellAktivitet` methods
5. **Processor Extractor** - Analyzes processor classes:
   - Extracts `doProcess()` and `onFinished()` methods
   - Parses conditional logic (if/when statements)
   - Detects feature toggle calls
   - Identifies end states (`aktivitetFullfort()`)
6. **Flow Graph Generator** - Builds and consolidates the flow diagram
7. **Graphviz Integration** - Converts DOT to requested format
8. **File Opener** - Automatically opens the generated visualization

### Data Structures

```rust
struct ClassInfo {
    name: String,                    // Class name
    file: PathBuf,                   // Source file path
    supertypes: Vec<String>,         // List of parent classes/interfaces
    initial_aktivitet: Option<String>, // Starting aktivitet from opprettInitiellAktivitet
}

struct ProcessorInfo {
    aktivitet_class: String,         // The aktivitet this processes
    processor_class: String,         // The processor class name
    next_aktiviteter: Vec<NextAktivitet>, // Possible next steps
}

struct NextAktivitet {
    aktivitet_name: String,          // Name of next aktivitet
    condition: Option<String>,       // Condition for this path (if any)
}
```

## Dependencies

- `tree-sitter` (0.22) - Parser infrastructure
- `tree-sitter-kotlin` (0.3) - Kotlin language grammar
- `walkdir` (2.4) - Directory traversal
- `anyhow` (1.0) - Error handling
- `clap` (4.5) - Command-line argument parsing
- `opener` (0.7) - Cross-platform file opening

## Troubleshooting

### "No .kt files found"
Make sure you're pointing to a directory that contains Kotlin source files. The tool searches recursively.

### "No Behandling classes with initial aktivitet found"
The tool looks for classes that:
1. Extend a class with "Behandling" in the name
2. Have an `opprettInitiellAktivitet()` method

Make sure your codebase follows this pattern.

### "Could not run graphviz 'dot' command"
Install Graphviz as described in the Prerequisites section. Verify with:
```bash
dot -V
```

### Graph doesn't open automatically
The graph only opens when you use the `--open` flag. If it still doesn't open, check that your system has a default application for the output format.

## Testing

Test data is included in `testdata/fleksibel_alderspensjon_sak_behandling/` for basic verification.

```bash
# Run with test data
behandling-flow testdata/fleksibel_alderspensjon_sak_behandling --verbose

# Run unit tests
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## For AI Agents

If you're an AI agent (LLM, coding assistant) working with this codebase, see [AGENTS.md](AGENTS.md) for a comprehensive technical guide including:
- Architecture overview
- Data structures and key functions
- Code patterns and conventions
- Common modifications and debugging tips
- Edge cases and best practices

## License

MIT