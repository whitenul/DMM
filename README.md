# Desk Manager

A Windows desktop application launcher and manager built with Tauri 2.0, Vue 3, and TypeScript.

## Features

- **Application Launcher**: Organize and launch your applications through a customizable categorized grid.
- **Theme System**: Full theme support with light/dark mode, custom accent colors, and built-in Fluent Design effects (Mica/Acrylic).
- **Transparency Control**: Adjustable window transparency with native DWM integration.
- **Background Images**: Set custom background images with blur effects.
- **Global Search**: Quickly find and launch applications with a global shortcut.
- **Drag & Drop**: Reorder categories and items via drag-and-drop.
- **Auto Scan**: Automatically scan and import applications from the Start Menu or custom folders.
- **Multi-language**: Supports Chinese (Simplified) and English.

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Framework | Tauri 2.0 |
| Frontend Framework | Vue 3 + TypeScript |
| State Management | Pinia |
| UI Style | Fluent Design (Mica/Acrylic) |
| Database | SQLite (rusqlite) |
| Build Tool | Vite |

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v20+)
- [pnpm](https://pnpm.io/) (v10+)
- [Rust](https://www.rust-lang.org/) (latest stable)

### Setup

```bash
# Install dependencies
pnpm install

# Start development server
pnpm tauri dev
```

### Build

```bash
# Build for production
pnpm tauri build
```

## Project Structure

```
Desk Manager/
├── src/                    # Frontend source
│   ├── assets/styles/      # Global CSS and design tokens
│   ├── components/         # Vue components
│   ├── composables/        # Vue composables
│   ├── pages/              # Page components
│   ├── router/             # Vue Router configuration
│   ├── stores/             # Pinia stores
│   ├── themes/             # Built-in theme definitions
│   ├── types/              # TypeScript type definitions
│   └── utils/              # Utility functions
├── src-tauri/              # Rust backend
│   ├── src/                # Main Rust source
│   ├── crates/             # Workspace crates
│   └── capabilities/       # Tauri capability files
└── ...
```

## License

ISC
