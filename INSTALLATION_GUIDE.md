# rfgrep Installation Guide

This guide provides comprehensive instructions for installing rfgrep v0.2.1, including the main program, man pages, and shell completions.

## Version Information

 **Current Version**: 0.2.1

## Quick Installation

### 1. Install rfgrep

```bash
# Via Cargo (recommended)
cargo install rfgrep

# From GitHub
cargo install --git https://github.com/rfgrep/rfgrep.git

# From source
git clone https://github.com/rfgrep/rfgrep.git
cd rfgrep
cargo build --release
```

### 2. Install Man Pages

```bash
# System-wide installation (requires sudo)
cd man
sudo make install

# User installation (no sudo required)
cd man
make install-user
```

### 3. Install Shell Completions

Choose your shell:

**Bash:**
```bash
rfgrep completions bash >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**
```bash
mkdir -p ~/.zsh/completions
rfgrep completions zsh > ~/.zsh/completions/_rfgrep
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
autoload -U compinit && compinit
```

**Fish:**
```bash
rfgrep completions fish --install --user
```

**PowerShell:**
```bash
rfgrep completions powershell > rfgrep-completion.ps1
. rfgrep-completion.ps1
```

## Detailed Installation

### Prerequisites

- Rust toolchain (1.70+)
- Cargo package manager
- Git (for source installation)

### Step-by-Step Installation

#### 1. Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### 2. Install rfgrep

**Option A: Via Cargo (Recommended)**
```bash
cargo install rfgrep
```

**Option B: From Source**
```bash
git clone https://github.com/rfgrep/rfgrep.git
cd rfgrep
cargo build --release
```

#### 3. Install Man Pages

**System-wide Installation:**
```bash
cd man
sudo make install
```

**User Installation:**
```bash
cd man
make install-user
```

Add to your shell profile (`.bashrc`, `.zshrc`, etc.):
```bash
export MANPATH=$MANPATH:$HOME/.local/share/man
```

#### 4. Install Shell Completions

**Bash:**
```bash
# Generate and source completion
rfgrep completions bash >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**
```bash
# Create completion directory
mkdir -p ~/.zsh/completions

# Generate completion file
rfgrep completions zsh > ~/.zsh/completions/_rfgrep

# Add to .zshrc
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
autoload -U compinit && compinit
```

**Fish:**
```bash
# Generate and install
rfgrep completions fish --install --user
```

**PowerShell:**
```bash
# Generate and import
rfgrep completions powershell > rfgrep-completion.ps1
. rfgrep-completion.ps1
```

## Verification

### Test Basic Functionality

```bash
# Test help
rfgrep --help

# Test search
rfgrep search "test" --extensions rs

# Test list
rfgrep list --extensions rs

# Test interactive mode
rfgrep interactive "test" --extensions rs
```

### Test Man Pages

```bash
# Test main man page
man rfgrep

# Test command-specific man pages
man rfgrep-search
man rfgrep-interactive
man rfgrep-list
man rfgrep-completions
```

### Test Shell Completions

```bash
# Bash/Zsh: Type 'rfgrep ' and press TAB
rfgrep <TAB>

# Fish: Type 'rfgrep ' and press TAB
rfgrep <TAB>
```

### Automated Testing

```bash
# Test shell completions
./test_completions.sh

# Test man pages
./test_man_pages.sh
```

## Troubleshooting

### Man Pages Not Found

```bash
# Check if man pages are installed
ls ~/.local/share/man/man1/rfgrep*

# Add to shell profile if needed
echo 'export MANPATH=$MANPATH:$HOME/.local/share/man' >> ~/.bashrc
source ~/.bashrc
```

### Completions Not Working

**Bash:**
```bash
# Check if completion is loaded
complete -p | grep rfgrep

# Manual installation
rfgrep completions bash >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**
```bash
# Check completion directory
ls ~/.zsh/completions/_rfgrep

# Reload completions
autoload -U compinit && compinit
```

**Fish:**
```bash
# Check if completion is installed
ls ~/.config/fish/completions/rfgrep.fish

# Manual installation
rfgrep completions fish > ~/.config/fish/completions/rfgrep.fish
```

### Performance Issues

```bash
# Use dry-run to preview
rfgrep search "pattern" --dry-run

# Skip binary files
rfgrep search "pattern" --skip-binary

# Limit file size
rfgrep search "pattern" --max-size 10

# Use specific extensions
rfgrep search "pattern" --extensions rs,py,js
```

## Features Available After Installation

### Man Pages
- Complete command reference
- Detailed option descriptions
- Practical examples
- Performance tips
- Troubleshooting guides

### Shell Completions
- Command completion (`rfgrep <TAB>`)
- Option completion (`rfgrep search --<TAB>`)
- Extension completion (`--extensions <TAB>`)
- File path completion (`src/<TAB>`)
- Algorithm completion (`--algorithm <TAB>`)
- Output format completion (`--output-format <TAB>`)

### Advanced Features
- Interactive search mode
- Multiple output formats (JSON, XML, HTML, Markdown)
- Multiple search algorithms (Boyer-Moore, Regex, Simple)
- Adaptive memory management
- Parallel processing
- Binary file detection

## Uninstallation

### Remove rfgrep
```bash
cargo uninstall rfgrep
```

### Remove Man Pages
```bash
cd man
make uninstall
```

### Remove Shell Completions

**Bash:**
```bash
# Remove from .bashrc
sed -i '/rfgrep completions bash/d' ~/.bashrc
```

**Zsh:**
```bash
rm ~/.zsh/completions/_rfgrep
```

**Fish:**
```bash
rm ~/.config/fish/completions/rfgrep.fish
```

## CI/CD Workflow Integration

### GitHub Actions

The project includes automated CI/CD workflows that:

1. **Build and Test**: Automatically builds and tests on multiple platforms
2. **Release Management**: Creates releases with proper versioning
3. **Documentation**: Generates and publishes documentation
4. **Quality Assurance**: Runs linting, formatting, and security checks

### Release Process

1. **Version Bumping**: Update version in `Cargo.toml` and `CHANGELOG.md`
2. **Testing**: Run comprehensive test suite
3. **Documentation**: Update man pages and installation guides
4. **Release**: Create GitHub release with assets
5. **Publishing**: Publish to crates.io

### Automated Testing

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Check code quality
cargo clippy --all-targets

# Format code
cargo fmt --all -- --check

# Test shell completions
./test_completions.sh

# Test man pages
./test_man_pages.sh
```

## Support

- **Documentation**: `man rfgrep`
- **Help**: `rfgrep --help`
- **Issues**: https://github.com/rfgrep/rfgrep/issues
- **Source**: https://github.com/rfgrep/rfgrep
- **Releases**: https://github.com/rfgrep/rfgrep/releases 