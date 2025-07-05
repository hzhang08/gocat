# gocat: Terminal Go Game SGF Viewer & Editor

**gocat** is a modern, interactive terminal tool for viewing and editing Go game records in SGF (Smart Game Format). It offers a powerful, keyboard-driven interface to navigate, annotate, and modify game records directly from your terminal, making it ideal for Go players and reviewers.

---

## Screenshot

![Screenshot of gocat in action](screenshot.png)

---

## Features
- Parse and process SGF files
- Command-line interface
- Modular Rust codebase

## Usage

### 1. Build the Project

```
cargo build --release
```

### 2. Run the Tool

Replace `<your_file.sgf>` with your actual SGF file:

```
cargo run --release -- <your_file.sgf>
```

### 3. Output

The tool will process the provided SGF file and output results to the console. For more details on available commands and options, run:

```
cargo run -- --help
```

## Notes
- Press **h** at any time to see all available commands.
- All `.sgf` files are ignored by git (see `.gitignore`).
- Requires Rust and Cargo installed. Download from [rust-lang.org](https://www.rust-lang.org/tools/install).

## Project Structure
- `src/` - Rust source code
- `Cargo.toml` - Project manifest
- `.gitignore` - Ignored files and folders

## License
GPL-3.0-only
