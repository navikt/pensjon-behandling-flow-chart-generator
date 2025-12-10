# Installation Guide

This guide will help you install the `behandling-flow` CLI tool on your system.

## Quick Install

### Using the install script (macOS/Linux)

```bash
cd behandling-flow
./install.sh
```

This will:
1. Build the release binary
2. Install it to `~/.local/bin/behandling-flow`
3. Guide you through adding it to your PATH if needed

### Manual Installation

#### 1. Install Prerequisites

**Rust (Required)**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Graphviz (Required for diagram generation)**
```bash
# macOS
brew install graphviz

# Ubuntu/Debian
sudo apt-get install graphviz

# Fedora/RHEL
sudo dnf install graphviz

# Windows (with Chocolatey)
choco install graphviz

# Windows (with Scoop)
scoop install graphviz
```

Verify Graphviz installation:
```bash
dot -V
```

#### 2. Build the Binary

```bash
cd behandling-flow
cargo build --release
```

The binary will be created at `target/release/behandling-flow`.

#### 3. Install to System

**Option A: Copy to a system directory (requires sudo)**
```bash
sudo cp target/release/behandling-flow /usr/local/bin/
```

**Option B: Copy to user directory (no sudo required)**
```bash
mkdir -p ~/.local/bin
cp target/release/behandling-flow ~/.local/bin/
```

Then add to your shell profile (`~/.bashrc`, `~/.zshrc`, `~/.profile`, etc.):
```bash
export PATH="$PATH:$HOME/.local/bin"
```

Reload your shell:
```bash
source ~/.bashrc  # or ~/.zshrc
```

**Option C: Use directly from build directory**
```bash
# Add an alias to your shell profile
alias behandling-flow='/path/to/behandling-flow/target/release/behandling-flow'
```

#### 4. Verify Installation

```bash
behandling-flow --version
behandling-flow --help
```

## Platform-Specific Notes

### macOS

If you get a security warning when running the binary:
1. Go to System Preferences → Security & Privacy
2. Click "Allow Anyway" for the behandling-flow binary
3. Run the command again

### Windows

On Windows, use PowerShell or WSL (Windows Subsystem for Linux) for the best experience.

**Using PowerShell:**
```powershell
# Build
cargo build --release

# Copy to a directory in PATH
Copy-Item target\release\behandling-flow.exe C:\Users\YourName\.local\bin\
```

Add `C:\Users\YourName\.local\bin` to your PATH environment variable.

### Linux

Make sure `~/.local/bin` is in your PATH. Most modern Linux distributions include it by default, but if not, add this to your `~/.bashrc` or `~/.profile`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Updating

To update to the latest version:

```bash
cd behandling-flow
git pull
cargo build --release
# Then copy the binary again as in step 3
```

## Uninstalling

Remove the binary from wherever you installed it:

```bash
# If installed to /usr/local/bin
sudo rm /usr/local/bin/behandling-flow

# If installed to ~/.local/bin
rm ~/.local/bin/behandling-flow

# Remove any aliases from shell profile
# Edit ~/.bashrc, ~/.zshrc, etc. and remove the behandling-flow alias/path
```

## Troubleshooting

### "cargo: command not found"
Install Rust: https://rustup.rs/

### "dot: command not found" when running behandling-flow
Install Graphviz as shown in the prerequisites section.

### Permission denied
Make sure the binary is executable:
```bash
chmod +x /path/to/behandling-flow
```

### Binary not found after installation
Make sure the installation directory is in your PATH:
```bash
echo $PATH
```

If not, add it as shown in the installation steps.

## Next Steps

Once installed, check out the [README.md](README.md) for usage examples and [USAGE.md](USAGE.md) for detailed documentation.

Quick start:
```bash
# Analyze current directory
behandling-flow

# Analyze a specific project
behandling-flow /path/to/kotlin/project

# Show help
behandling-flow --help
```
