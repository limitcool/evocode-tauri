# evocode-tauri

A Tauri desktop application that provides a bridge service for [evocode](https://github.com/evolutions-code/evocode), enabling AI code assistant capabilities through a local proxy server.

## Features

- **Bridge Service**: Runs a local HTTP server (default: `127.0.0.1:17761`) to proxy AI API requests
- **Multi-Provider Support**: Configure multiple AI providers (OpenAI, Anthropic, etc.)
- **System Tray**: Runs in background with system tray icon and menu
- **Config Management**: Edit provider configurations through the app UI
- **Log Viewer**: Monitor bridge activity and debug issues
- **Cross-Platform**: Built with Tauri 2 for Windows, macOS, and Linux

## Installation

Download the latest release from the [GitHub Releases](https://github.com/evolutions-code/evocode-tauri/releases) page.

### Windows

Download and run the `.exe` installer or portable `.msi` file.

### macOS

Download the `.dmg` file and drag it to Applications.

### Linux

Download the `.AppImage` or `.deb` package.

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite + Ant Design Vue
- **Backend**: Rust + Tauri 2
- **Protocol**: evocode-proto (OpenAI-compatible wire protocol)

## Prerequisites (Development)

- Node.js 18+
- Rust 1.70+
- Yarn

## Development

```bash
# Install dependencies
yarn install

# Run in development mode
yarn tauri dev
```

## Build

```bash
# Build for production
yarn tauri build
```

## Configuration

The app reads configuration from `~/.evocode/config.toml`. Example:

```toml
model = "codex"

[providers.openai]
base_url = "https://api.openai.com/v1"
api_key = "your-api-key"
models = ["gpt-4", "gpt-4-turbo"]

[providers.anthropic]
base_url = "https://api.anthropic.com/v1"
api_key = "your-api-key"
models = ["claude-3-opus", "claude-3-sonnet"]
```

## Project Structure

```
├── src/ # Vue frontend source
│   ├── api/                  # Tauri bridge API calls
│   ├── components/           # Vue components
│   ├── composables/          # Vue composables (hooks)
│   ├── layouts/              # Page layouts
│   ├── router/               # Vue Router config
│   ├── stores/               # Pinia state stores
│   ├── types/                # TypeScript type definitions
│   ├── utils/                # Utility functions
│   └── views/ # Page views
├── src-tauri/                # Rust backend source
│   └── src/
│       ├── lib.rs            # Main library with Tauri commands
│       └── main.rs           # Application entry point
└── public/                   # Static assets
```