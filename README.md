# 🔒 PDF Encrypt

A simple desktop tool to password-protect PDF files. Built with [Tauri v2](https://v2.tauri.app/) + Rust.

![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- Select one or more PDF files
- Set a password
- Encrypts with AES-256 and saves as `filename_encrypted.pdf`
- Cross-platform (Windows, macOS, Linux)

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- **[qpdf](https://github.com/qpdf/qpdf)** — must be installed and on your `PATH`

### Install qpdf

| OS | Command |
|---|---|
| macOS | `brew install qpdf` |
| Ubuntu/Debian | `sudo apt install qpdf` |
| Windows | `choco install qpdf` or download from [GitHub releases](https://github.com/qpdf/qpdf/releases) |

## Development

```bash
# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Run in development mode
cd src-tauri
cargo tauri dev
```

## Build

```bash
cargo tauri build
```

The installer/binary will be in `src-tauri/target/release/bundle/`.

## How It Works

The app uses `qpdf` under the hood to apply AES-256 encryption to your PDF files. The Tauri frontend provides a clean drag-and-drop interface, while the Rust backend handles the encryption via `qpdf`.

## License

[MIT](LICENSE)
