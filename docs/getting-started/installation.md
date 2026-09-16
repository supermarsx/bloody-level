# Installation

## Requirements

- Windows, macOS, or Linux supported by Tauri 2.
- Node.js 20 or newer and npm.
- Rust through `rustup`, with a toolchain compatible with the repository's `rust-toolchain.toml` (currently Rust 1.95 or newer).
- The native build prerequisites listed by [Tauri's prerequisites guide](https://v2.tauri.app/start/prerequisites/).

The application runs locally after it is built. A first build may need internet access to download npm packages and a matching PDFium archive.

## Install and run

From the repository root:

```bash
npm ci
npm run tauri:dev
```

The Vite frontend alone can be started with:

```bash
npm run dev
```

Use the Tauri development command when you need the desktop shell, Rust commands, encrypted storage, or native file dialogs.

## PDFium

PDFium is the primary PDF text-extraction backend. The Rust build helper tries to download the target-matched archive into `src-tauri/binaries`. If that download is unavailable, the build continues with a warning so a manually supplied matching library can be used.

When a build reports that PDFium is missing, see [troubleshooting](../reference/troubleshooting.md). Do not copy a library built for another operating system or architecture.

## Optional OCR and models

Tesseract OCR is an optional fallback for reports that contain little extractable text. It must be compiled/enabled and available in the local environment before it can be used. The application does not silently download OCR or language-model assets at runtime.

The experimental olmOCR-2 and Phi-4 paths currently validate configuration and lifecycle state; they are not documented as a complete extraction guarantee. See [implementation status](../reference/status.md).

## Build a release bundle

```bash
npm run tauri:build
```

Release builds are platform and architecture specific. Read [releases and versioning](../developer/releases.md) before publishing an artifact.
