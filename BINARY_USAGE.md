# Binary Usage Reference

This document provides a complete reference for using the `behandling-flow` binary.

## Installation

See [INSTALL.md](INSTALL.md) for full installation instructions.

### Quick Install

```bash
cd behandling-flow
cargo build --release
sudo cp target/release/behandling-flow /usr/local/bin/
```

Or use the install script:
```bash
./install.sh
```

## Command Syntax

```
behandling-flow [OPTIONS] [PATH]
```

## Arguments

### `[PATH]`
- **Description**: Path to the Kotlin project directory
- **Default**: Current directory (`.`)
- **Type**: Optional positional argument
- **Examples**:
  ```bash
  behandling-flow                                    # Use current directory
  behandling-flow .                                  # Explicitly use current directory
  behandling-flow /path/to/kotlin/project           # Analyze specific directory
  behandling-flow ~/projects/my-kotlin-app          # Use home directory path
  ```

## Options

### `-f, --format <FORMAT>`
- **Description**: Output format for the generated graph
- **Default**: `svg`
- **Supported formats**: `svg`, `png`, `pdf`, `jpg`, `gif`, `ps`, and any other format supported by Graphviz
- **Examples**:
  ```bash
  behandling-flow --format svg      # Generate SVG (default)
  behandling-flow --format png      # Generate PNG
  behandling-flow --format pdf      # Generate PDF
  behandling-flow -f jpg            # Short form
  ```

### `--open`
- **Description**: Automatically open the generated graph
- **Default**: Graph is not opened (you must specify this flag to open)
- **Type**: Flag (no value needed)
- **Use case**: When you want to immediately view the generated diagram
- **Examples**:
  ```bash
  behandling-flow --open
  ```

### `-k, --keep-dot`
- **Description**: Keep the intermediate `.dot` file after conversion
- **Default**: `.dot` file is deleted after successful conversion
- **Type**: Flag (no value needed)
- **Use case**: When you want to manually edit the DOT file or debug the graph
- **Examples**:
  ```bash
  behandling-flow --keep-dot
  behandling-flow -k
  ```

### `-o, --output-dir <OUTPUT_DIR>`
- **Description**: Output directory for generated files
- **Default**: Current working directory
- **Type**: Optional path argument
- **Note**: Directory will be created if it doesn't exist
- **Examples**:
  ```bash
  behandling-flow --output-dir ./graphs
  behandling-flow --output-dir /tmp/flow-diagrams
  behandling-flow -o ~/Documents/diagrams
  ```

### `-v, --verbose`
- **Description**: Show detailed analysis information
- **Default**: Minimal output
- **Type**: Flag (no value needed)
- **Use case**: When you want to see the complete flow analysis, class index, and processor details
- **Examples**:
  ```bash
  behandling-flow --verbose
  behandling-flow -v
  ```

### `-h, --help`
- **Description**: Print help information
- **Type**: Flag (no value needed)
- **Examples**:
  ```bash
  behandling-flow --help
  behandling-flow -h
  ```

### `-V, --version`
- **Description**: Print version information
- **Type**: Flag (no value needed)
- **Examples**:
  ```bash
  behandling-flow --version
  behandling-flow -V
  ```

## Usage Examples

### Basic Usage

```bash
# Analyze current directory, generate SVG, and open it
behandling-flow

# Analyze specific directory
behandling-flow /path/to/kotlin/project

# Use verbose output to see detailed analysis
behandling-flow /path/to/project --verbose
```

### Output Format Control

```bash
# Generate PNG instead of SVG
behandling-flow --format png

# Generate PDF for documentation
behandling-flow --format pdf

# Generate high-resolution PNG
behandling-flow --format png
```

### File Management

```bash
# Generate and open
behandling-flow --open

# Keep the DOT file for manual editing
behandling-flow --keep-dot

# Save to specific directory
behandling-flow --output-dir ./documentation/diagrams

# Combine options
behandling-flow --format png --keep-dot --output-dir ./output --open
```

### Advanced Usage

```bash
# Generate multiple formats (requires separate runs)
behandling-flow --format svg --output-dir ./output
behandling-flow --format png --output-dir ./output
behandling-flow --format pdf --output-dir ./output

# Analyze and save without opening (default behavior, good for CI/CD)
behandling-flow /path/to/project \
  --format png \
  --output-dir ./build/diagrams

# Debug mode with all artifacts
behandling-flow /path/to/project \
  --verbose \
  --keep-dot \
  --output-dir ./debug
```

## Output Files

The tool generates files named after the Behandling class:

- `{BehandlingName}_flow.{format}` - The main output file
- `{BehandlingName}_flow.dot` - Intermediate DOT file (if `--keep-dot` is used)

Example:
```
FleksibelApSakBehandling_flow.svg
FleksibelApSakBehandling_flow.dot  (with --keep-dot)
```

## Exit Codes

- `0` - Success
- `1` - Error (with error message on stderr)

## Environment Variables

None currently used. All configuration is via command-line options.

## Adding to PATH

### macOS/Linux

Add to `~/.bashrc`, `~/.zshrc`, or `~/.profile`:
```bash
export PATH="$PATH:/path/to/behandling-flow/target/release"
```

Or copy to a system directory:
```bash
sudo cp target/release/behandling-flow /usr/local/bin/
```

### Windows

**PowerShell:**
```powershell
$env:PATH += ";C:\path\to\behandling-flow\target\release"
```

Or add to system PATH via System Properties → Environment Variables.

## Integration Examples

### Shell Script

```bash
#!/bin/bash
set -e

PROJECT_DIR="/path/to/kotlin/project"
OUTPUT_DIR="./diagrams"

echo "Generating behandling flow diagram..."
behandling-flow "$PROJECT_DIR" \
  --format svg \
  --output-dir "$OUTPUT_DIR" \
  --no-open

echo "Diagram generated at: $OUTPUT_DIR"
```

### Makefile

```makefile
.PHONY: diagram
diagram:
	behandling-flow src/main/kotlin \
		--format svg \
		--output-dir docs/diagrams \
		--no-open

.PHONY: diagram-all
diagram-all:
	behandling-flow src/main/kotlin --format svg --output-dir docs --no-open
	behandling-flow src/main/kotlin --format png --output-dir docs --no-open
	behandling-flow src/main/kotlin --format pdf --output-dir docs --no-open
```

### GitHub Actions

```yaml
name: Generate Flow Diagram

on:
  push:
    branches: [ main ]

jobs:
  diagram:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Graphviz
        run: sudo apt-get install -y graphviz
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build behandling-flow
        run: |
          cd behandling-flow
          cargo build --release
      
      - name: Generate diagram
        run: |
          behandling-flow/target/release/behandling-flow . \
            --format svg \
            --output-dir ./diagrams \
            --no-open
      
      - name: Upload diagram
        uses: actions/upload-artifact@v3
        with:
          name: flow-diagram
          path: diagrams/
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Generate updated flow diagram before commit
behandling-flow . \
  --format svg \
  --output-dir docs \
  --no-open

# Stage the generated diagram
git add docs/*_flow.svg
```

## Troubleshooting

### "No .kt files found"
- Verify the path is correct
- Make sure the directory contains Kotlin source files
- Use `--verbose` to see what files were scanned

### "No Behandling classes with initial aktivitet found"
- Ensure your code has classes that extend `Behandling`
- Make sure those classes have `opprettInitiellAktivitet()` method
- Use `--verbose` to see what classes were indexed

### "dot: command not found"
- Install Graphviz: `brew install graphviz` (macOS) or `sudo apt install graphviz` (Linux)
- Verify with: `dot -V`

### "Could not automatically open file"
- The file was still generated, just not opened
- Open manually from the output directory
- This only happens when using the `--open` flag

### Graph is incomplete or has "?" nodes
- Some processors might be missing or not following expected patterns
- Use `--verbose` to see which processors were found
- Use `--keep-dot` to inspect the DOT file

## Performance

Typical performance for different project sizes:

| Files | Classes | Time   |
|-------|---------|--------|
| 10    | 50      | < 1s   |
| 50    | 200     | 1-2s   |
| 100   | 500     | 2-5s   |
| 500   | 2000    | 10-20s |

The tool is fast enough to run interactively even on large codebases.

## See Also

- [README.md](README.md) - Overview and features
- [INSTALL.md](INSTALL.md) - Installation guide
- [USAGE.md](USAGE.md) - Detailed usage documentation
- [QUICKSTART.md](QUICKSTART.md) - Quick start guide