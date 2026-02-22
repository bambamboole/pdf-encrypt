# 🔒 PDF Encrypt

A simple desktop tool to password-protect PDF files. Built with [Tauri v2](https://v2.tauri.app/) + Rust.

![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)

![PDF Encrypt Screenshot](screenshot.png)

## Features

- Select one or more PDF files
- Set a password
- Encrypts with AES-256 and saves as `filename_encrypted.pdf`
- Cross-platform (Windows, macOS, Linux)
- No external dependencies — encryption is handled natively in Rust via [lopdf](https://github.com/J-F-Liu/lopdf)

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (for Tauri CLI)

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

The app uses the [lopdf](https://github.com/J-F-Liu/lopdf) Rust library to apply AES-256 encryption to your PDF files. The Tauri frontend provides a clean file-selection interface, while the Rust backend handles the encryption entirely in-process — no external tools required.

## License

[MIT](LICENSE)
