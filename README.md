# rfgrep  

A command-line utility for recursively searching and listing files with advanced filtering capabilities. Built in Rust.

[<img alt="crates.io" src="https://img.shields.io/crates/v/rfgrep.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/rfgrep)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rfgrep-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/rfgrep)
[![CI](https://github.com/kh3rld/rfgrep/actions/workflows/ci.yml/badge.svg)](https://github.com/kh3rld/rfgrep/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/kh3rld/rfgrep)](https://github.com/kh3rld/rfgrep/blob/main/LICENSE)

[![DeepSource](https://app.deepsource.com/gh/kh3rld/rfgrep.svg/?label=active+issues&show_trend=true)](https://app.deepsource.com/gh/kh3rld/rfgrep/)


[![Get it from the Snap Store](https://snapcraft.io/en/dark/install.svg)](https://snapcraft.io/rfgrep)

## Features

- **Advanced Search**
  - Regex, plain text, and whole-word matching
  - Recursive directory traversal
  - Binary file detection
  - Extension filtering
  - Size limits

- **File Listing**
  - Detailed/simple output formats
  - Extension statistics
  - Size filtering
  - Hidden file handling

- **Utilities**
  - Clipboard copy support
  - Dry-run mode
  - Logging to file
  - Progress indicators

- **Unix Pipeline Integration**
  - Stdin support for piped input
  - Automatic quiet mode when piped
  - Seamless integration with grep, awk, sed, xargs
  - Count-only mode (`-c`)
  - Files-with-matches mode (`-l`)

- **Output Formats** (v0.4.0)
  - CSV export for spreadsheet analysis
  - TSV export for tab-separated data
  - JSON, XML, HTML, Markdown formats
  - NDJSON for streaming JSON

<!-- BENCHMARK_RESULTS_START -->
## Performance Benchmarks

Benchmark results will be automatically updated here when CI runs.
![Benchmark Flamegraph](results/benchmark_flamegraph.svg)
<!-- BENCHMARK_RESULTS_END -->

## Installation

Assuming you have [Rust installed][Rust], run:

[Rust]: https://www.rust-lang.org/

### Via Cargo

```bash
cargo install rfgrep
```

### From GitHub
```bash
cargo install --git https://github.com/kh3rld/rfgrep.git
```

### From Source

```bash
git clone https://github.com/kh3rld/rfgrep.git
cargo build --release
```

### Installing Man Pages

After installing rfgrep, you can install the comprehensive man pages:

#### System-wide Installation (requires sudo)
```bash
cd man
sudo make install
```

#### User Installation (no sudo required)
```bash
cd man
make install-user
```

Then add to your shell profile (`.bashrc`, `.zshrc`, etc.):
```bash
export MANPATH=$MANPATH:$HOME/.local/share/man
```

### Installing Shell Completions

rfgrep supports tab completion for all major shells:

#### Bash
```bash
# Generate and source completion
rfgrep completions bash >> ~/.bashrc
source ~/.bashrc
```

#### Zsh
```bash
# Generate completion file
rfgrep completions zsh > ~/.zsh/completions/_rfgrep
# Add to .zshrc
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
autoload -U compinit && compinit
```

#### Fish
```bash
# Generate and install
rfgrep completions fish --install --user
```

#### PowerShell
```bash
# Generate and import
rfgrep completions powershell > rfgrep-completion.ps1
. rfgrep-completion.ps1
```

## Usage

### Basic Search

```bash
rfgrep search "pattern"
```

### Search with Options

```bash
rfgrep search "pattern" \
    --mode regex \
    --extensions rs,toml \
    --max-size 5 \
    --skip-binary \
    --copy
```

### File Listing

```bash
# Simple list
rfgrep list

# Detailed view
rfgrep list --long --recursive

# With filters
rfgrep list --extensions rs,toml --max-size 10 --show-hidden
```

## Documentation

See DESIGN_OPTIMIZATION.md for the latest simulation findings and the optimized framework proposal, including cross-disciplinary applications and roadmap.

### Simulations

Quickly run built-in simulations and write a CSV report to ./results/simulations.csv:

```bash
rfgrep simulate
```

You can change the working directory with --path to select a corpus (defaults to . and prefers ./bench_data if present).

### Man Pages

After installation, comprehensive man pages are available:

```bash
# Main man page
man rfgrep

# Command-specific man pages
man rfgrep-search
man rfgrep-interactive
man rfgrep-list
man rfgrep-completions
```

The man pages include:
- Complete command reference
- Detailed option descriptions
- Practical examples
- Performance tips
- Troubleshooting guides

### Shell Completions

Once installed, tab completion provides:
- Command completion (`rfgrep <TAB>`)
- Option completion (`rfgrep search --<TAB>`)
- Extension completion (`--extensions <TAB>`)
- File path completion (`src/<TAB>`)

### Troubleshooting

#### Man Pages Not Found
```bash
# Check if man pages are installed
ls ~/.local/share/man/man1/rfgrep*

# Add to shell profile if needed
echo 'export MANPATH=$MANPATH:$HOME/.local/share/man' >> ~/.bashrc
```

#### Completions Not Working
```bash
# Regenerate completions
rfgrep completions bash > ~/.bash_completion.d/rfgrep

# Reload shell configuration
source ~/.bashrc

# For zsh, ensure completion directory exists
mkdir -p ~/.zsh/completions
rfgrep completions zsh > ~/.zsh/completions/_rfgrep

# For fish, install to user directory
rfgrep completions fish --install --user
```

#### Performance Issues
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

#### Shell-Specific Troubleshooting

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

## Command Reference

### Global Options

| Option       | Description                     |
|--------------|---------------------------------|
| `--log PATH` | Write logs to specified file    |
| `--path DIR` | Base directory (default: `.`)   |

### Search Command

| Option                       | Description                                                        |
|------------------------------|--------------------------------------------------------------------|
| `--mode MODE`                | Search mode: regex/text/word                                       |
| `--extensions EXT`           | Comma-separated file extensions                                    |
| `--max-size MB`              | Skip files larger than specified MB                                |
| `--skip-binary`              | Skip binary files                                                  |
| `--dry-run`                  | Preview files without processing                                   |
| `--copy`                     | Copy results to clipboard                                          |
| `--quiet`, `-q`              | Suppress non-essential output (v0.4.0)                             |
| `--count`, `-c`              | Show only count of matches (v0.4.0)                                |
| `--files-with-matches`, `-l` | Show only filenames with matches (v0.4.0)                          |
| `--output-format`            | Output format: text/json/csv/tsv/xml/html/markdown                 |
| `--ndjson`                   | Output newline-delimited JSON (v0.4.0)                             |
| `--safety-policy`            | Safety policy: default/conservative/performance                    |
| `--threads N`                | Number of threads for parallel processing                          |
| `--file-types`               | File type strategy: default/comprehensive/conservative/performance |
| `--include-extensions`       | Override to include specific file types                            |
| `--exclude-extensions`       | Override to exclude specific file types                            |
| `--search-all-files`         | Search all file types (comprehensive mode)                         |
| `--text-only`                | Only search text files (conservative mode)                         |

### List Command

| Option             | Description                         |
|--------------------|-------------------------------------|
| `--extensions EXT` | Comma-separated file extensions     |
| `--long`           | Detailed output format              |
| `--recursive`      | Recursive directory traversal       |
| `--show-hidden`    | Include hidden files/directories    |
| `--max-size MB`    | Skip files larger than specified MB |
| `--skip-binary`    | Skip binary files                   |

## Examples

1. Find all Rust files containing "HashMap":

```bash
rfgrep search "HashMap" --extensions rs
```

2. List all Markdown files under 1MB:

```bash
rfgrep list --extensions md --max-size 1
```

3. Search with regex and copy to clipboard:

```bash
rfgrep search "fn\s+\w+\s*\(" --mode regex --copy
```

4. **New in v0.4.0:** Count-only mode (like `grep -c`):

```bash
rfgrep search "error" --extensions log -c
# Output: 42
```

5. **New in v0.4.0:** Files-with-matches mode (like `grep -l`):

```bash
rfgrep search "TODO" --extensions rs -l
# Output: List of files containing TODO
```

6. **New in v0.4.0:** CSV export for analysis:

```bash
rfgrep search "HashMap" --extensions rs --output-format csv > results.csv
```

7. **New in v0.4.0:** Unix pipeline integration:

```bash
# Count matches per file
rfgrep search "error" --output-format csv | awk -F',' 'NR>1 {print $1}' | sort | uniq -c

# Process files with xargs
rfgrep list --extensions rs | xargs wc -l

# Filter with grep
rfgrep search "function" | grep "async"
```

8. **New in v0.4.0:** Stdin/Pipe support:

```bash
# Pipe input from another command
cat file.log | rfgrep search "error"

# Chain with other Unix tools
cat .zsh_history | rfgrep search "git" -c

# Process command output
echo "test data with error" | rfgrep search "error"

# Complex pipeline
cat *.log | rfgrep search "WARNING\|ERROR" --mode regex -c
```

9. Advanced file type control:

```bash
rfgrep search "pattern" --file-types comprehensive --include-extensions pdf,docx
rfgrep search "pattern" --text-only --safety-policy conservative
rfgrep search "pattern" --threads 4 --safety-policy performance
```

10. Simulation and benchmarking:

```bash
rfgrep simulate
```

## Performance Tips

- Use `--skip-binary` to avoid unnecessary file checks
- Limit scope with `--extensions` and `--max-size`
- For large directories, `--dry-run` first to preview
- Use `--safety-policy performance` for faster processing
- Adjust `--threads` based on your CPU cores
- Use `--file-types conservative` for safe text-only search

## Advanced Usage

### Interactive Mode
```bash
# Start interactive search
rfgrep interactive "pattern"

# Interactive search with specific algorithm
rfgrep interactive "pattern" --algorithm boyer-moore

# Interactive search in specific file types
rfgrep interactive "pattern" --extensions rs,py
```

### Output Formats
```bash
# JSON output for programmatic processing
rfgrep search "pattern" --output-format json

# NDJSON (newline-delimited JSON) for streaming
rfgrep search "pattern" --ndjson

# CSV output for spreadsheet analysis (v0.4.0)
rfgrep search "pattern" --output-format csv

# TSV output for tab-separated data (v0.4.0)
rfgrep search "pattern" --output-format tsv

# XML output for structured data
rfgrep search "pattern" --output-format xml

# HTML output for web display
rfgrep search "pattern" --output-format html

# Markdown output for documentation
rfgrep search "pattern" --output-format markdown
```

### Search Algorithms
```bash
# Boyer-Moore (fast for plain text)
rfgrep search "pattern" --algorithm boyer-moore

# Regular expression
rfgrep search "pattern" --algorithm regex

# Simple linear search
rfgrep search "pattern" --algorithm simple
```

## Verification

### Test Man Pages
```bash
# Verify man pages are accessible
man rfgrep
man rfgrep-search
man rfgrep-interactive
man rfgrep-list
man rfgrep-completions
```

### Test Shell Completions
```bash
# Bash: Type 'rfgrep ' and press TAB
rfgrep <TAB>

# Zsh: Type 'rfgrep ' and press TAB
rfgrep <TAB>

# Fish: Type 'rfgrep ' and press TAB
rfgrep <TAB>
```

### Test Basic Functionality
```bash
# Test search functionality
rfgrep search "test" --extensions rs

# Test list functionality
rfgrep list --extensions rs

# Test interactive mode
rfgrep interactive "test" --extensions rs
```

### Clipboard behavior in CI/headless environments

Note: the `--copy` option attempts to use the system clipboard and may fail in headless CI environments (X11/Wayland not available). In those environments run without `--copy` or provide a virtual display (Xvfb) or configure your CI to provide a clipboard service. The application will log a warning if the clipboard operation times out.

### Automated Testing
```bash
# Test shell completions
./test_completions.sh

# Test man pages
./test_man_pages.sh
```

## Contributing

Contributions are welcome! Please open an issue or PR for any:
- Bug reports
- Feature requests
- Performance improvements
