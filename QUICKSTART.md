# Behandling Flow Analyzer - Quick Start

Get up and running in 2 minutes! 🚀

## 1. Install Prerequisites

```bash
# Install Graphviz (required for diagram generation)
brew install graphviz          # macOS
sudo apt install graphviz      # Linux
choco install graphviz         # Windows
```

## 2. Build the Tool

```bash
cd behandling-flow
cargo build --release
```

## 3. Add to PATH (Optional but Recommended)

```bash
# Copy to a directory in your PATH
sudo cp target/release/behandling-flow /usr/local/bin/

# OR add an alias to your shell profile
echo 'alias behandling-flow="/path/to/behandling-flow/target/release/behandling-flow"' >> ~/.zshrc
source ~/.zshrc
```

## 4. Run It!

```bash
# Analyze current directory
behandling-flow

# Analyze a specific directory
behandling-flow /path/to/kotlin/project

# Get help
behandling-flow --help
```

## Common Use Cases

### Just generate the diagram (default)
```bash
behandling-flow /path/to/project
```
✨ Generates SVG file

### Generate PNG instead of SVG
```bash
behandling-flow /path/to/project --format png
```

### Generate and open automatically
```bash
behandling-flow /path/to/project --open
```

### Show condition labels
```bash
# Default: Clean graphs without condition labels
behandling-flow /path/to/project

# Show conditions on edges (e.g., "harData", "NOT (isValid())")
behandling-flow /path/to/project --show-conditions
```

### Show legend
```bash
# Default: No legend (cleaner)
behandling-flow /path/to/project

# Show color legend in graph
behandling-flow /path/to/project --show-legend
```

### Use different edge styles
```bash
# Straight lines (default)
behandling-flow /path/to/project --edge-style straight

# Curved edges
behandling-flow /path/to/project --edge-style curved

# Orthogonal (right-angle only) edges
behandling-flow /path/to/project --edge-style ortho
```

### Save diagrams to a specific folder
```bash
behandling-flow /path/to/project --output-dir ./diagrams
```

### Keep the DOT file for manual editing
```bash
behandling-flow /path/to/project --keep-dot
```

### See detailed analysis output
```bash
behandling-flow /path/to/project --verbose
```

### Multiple options at once
```bash
behandling-flow /path/to/project \
  --format pdf \
  --edge-style straight \
  --show-conditions \
  --show-legend \
  --output-dir ./output \
  --keep-dot \
  --verbose
```

## What You'll Get

The tool will:
1. ✅ Scan your Kotlin files
2. ✅ Find all Behandling classes
3. ✅ Extract aktivitet flows
4. ✅ Generate a beautiful diagram
5. ✅ Optionally open it (with --open flag)

## Output Formats Supported

- `svg` (default) - Scalable vector graphics, best for viewing
- `png` - Raster image, good for presentations
- `pdf` - PDF document, good for printing
- `dot` - Raw DOT file (with `--keep-dot`)
- And any other format supported by Graphviz!

## Edge Styles Supported

- `straight` (default) - Straight line segments, clean and modern
- `curved` - Smooth bezier curves, flowing appearance
- `ortho` - Strictly orthogonal (right angles only), very structured

## Condition Labels

By default, graphs are **clean without condition labels** for better visual overview.

Use `--show-conditions` to see:
- Actual Kotlin conditions (e.g., `harData`, `NOT (isValid())`)
- Feature toggle flags marked with 🚩
- No "else" or "alternative paths" clutter - just the meaningful conditions

## Example Output

```
🔍 Scanning directory: /path/to/project
📄 Scanned 54 .kt files
📚 Indexed 122 classes
⚙️  Found 27 processors

📊 Generating graphs...
  ✅ Generated: FleksibelApSakBehandling_flow.svg

✨ Done!
```

## Understanding the Diagram

### Node Colors
- 🟢 **Green** - START (entry point)
- 🟣 **Purple** - Important AldeAktivitet
- 🔵 **Blue** - Regular processing
- 🟡 **Gold** - Waiting/pause activities
- 🔴 **Red** - Manual intervention required
- 🟥 **Dark Red** - Abort/rejection
- 🟩 **Green** - Decision/execution
- 🩷 **Pink** - END (terminal)

### Edge Labels
- Show conditional logic from your code
- 🚩 Feature toggles are marked
- Multiple paths are consolidated (e.g., "7 alternative paths")

## Troubleshooting

### "No .kt files found"
Make sure you're pointing to the right directory.

### "dot: command not found"
Install Graphviz (see step 1).

### "No Behandling classes found"
Your project might not follow the expected pattern. Use `--verbose` to see what was found.

## Next Steps

- 📖 Read the full [README.md](README.md) for detailed information
- 📚 Check [USAGE.md](USAGE.md) for advanced features
- 💻 See [INSTALL.md](INSTALL.md) for installation options

## Need Help?

```bash
behandling-flow --help
```

That's it! Happy analyzing! 🎉