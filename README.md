# Matrix Client

A desktop Matrix protocol client built with Tauri, React, and TypeScript. This application provides a native desktop experience for Matrix communication using modern web technologies with Rust-powered backend.

## Features

- Native desktop application powered by Tauri
- React-based user interface
- TypeScript for type-safe development
- Cross-platform support (Windows, macOS, Linux)
- Lightweight and fast performance

## Tech Stack

- **Frontend**: React 19, TypeScript
- **Desktop Framework**: Tauri 2.x
- **Build Tool**: Vite 7.x
- **Language**: TypeScript 5.8

## Prerequisites

Before you begin, ensure you have the following installed:

- [Node.js](https://nodejs.org/) (v18 or higher)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Tauri Prerequisites](https://tauri.app/v2/guides/prerequisites) for your operating system

## Installation

1. Clone the repository:
```bash
git clone https://github.com/thomasPeelmanUCLL/matrix-client.git
cd matrix-client
```

2. Install dependencies:
```bash
npm install
```

## Development

### Running the Development Server

To start the application in development mode:

```bash
npm run tauri dev
```

This will launch both the Vite development server and the Tauri application window with hot-reload enabled.

### Building the Frontend Only

To run just the web development server:

```bash
npm run dev
```

Then open your browser to `http://localhost:5173`

## Building for Production

### Build the Application

To create a production build:

```bash
npm run tauri build
```

This will:
1. Compile TypeScript and bundle the frontend with Vite
2. Compile the Rust backend
3. Create platform-specific installers in `src-tauri/target/release/bundle/`

### Preview Production Build

To preview the production frontend build:

```bash
npm run build
npm run preview
```

## Project Structure

```
matrix-client/
├── src/                  # React application source code
├── src-tauri/            # Tauri/Rust backend code
│   ├── src/              # Rust source files
│   ├── icons/            # Application icons
│   └── Cargo.toml        # Rust dependencies
├── public/               # Static assets
├── index.html            # HTML entry point
├── package.json          # Node.js dependencies and scripts
├── tsconfig.json         # TypeScript configuration
└── vite.config.ts        # Vite build configuration
```

## Available Scripts

- `npm run dev` - Start Vite development server
- `npm run build` - Build frontend for production
- `npm run preview` - Preview production build
- `npm run tauri` - Run Tauri CLI commands
- `npm run tauri dev` - Start Tauri development mode
- `npm run tauri build` - Build production application

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) with extensions:
  - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
  - [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint)
  - [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Resources

- [Tauri Documentation](https://tauri.app/)
- [React Documentation](https://react.dev/)
- [Matrix Protocol](https://matrix.org/)
- [TypeScript Documentation](https://www.typescriptlang.org/)
- [Vite Documentation](https://vitejs.dev/)