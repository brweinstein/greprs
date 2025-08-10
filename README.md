# greprs

A fast grep clone written in Rust with parallel processing and modern features.

## Features

- **Pattern matching**: Basic regex, fixed strings, word/line boundaries
- **Search options**: Case-insensitive, recursive directories, inverted matching  
- **Output controls**: Line numbers, match counts, filename display
- **Performance**: Parallel file processing for speed

## Quick Start

```bash
# Search for "error" in logs directory
greprs "error" ./logs

# Case-insensitive search with line numbers
greprs -in "TODO" ./src

# Count matches in all Rust files
greprs -c "panic!" ./src

# Find files containing "deprecated"
greprs -l "deprecated" ./
```

## Installation

```bash
git clone https://github.com/brweinstein/greprs.git
cd greprs
cargo install --path .
```

## Usage

```bash
greprs [OPTIONS] <pattern> <files...>
```

**Common options:**
- `-i` - ignore case
- `-r` - search directories recursively  
- `-n` - show line numbers
- `-c` - count matches only
- `-l` - list files with matches
- `-v` - invert match (show non-matching lines)
- `-F` - treat pattern as literal string

## Performance

Currently ~38% slower than GNU grep on small workloads, but actively being optimized.

**Benchmark (25 files, 25K lines):**
```
Tool     Time (ms)    Memory (MB)    Matches
grep         5.3           0.5          258  
greprs       7.3           0.8          258
```

Run your own benchmarks:
```bash
./benchmark/compare.py --greprs-bin target/release/greprs --workload small
```

## Development

```bash
# Build and test
cargo build --release
cargo test

# Run locally  
cargo run -- "pattern" ./files
```

---

MIT License © 2025 Ben Weinstein