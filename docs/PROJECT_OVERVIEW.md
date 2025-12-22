# Matrix Client - Project Overview

## Table of Contents
1. [Introduction](#introduction)
2. [Architecture](#architecture)
3. [Technology Stack](#technology-stack)
4. [Project Structure](#project-structure)
5. [Key Features](#key-features)
6. [Development Workflow](#development-workflow)
7. [Security & Encryption](#security--encryption)

## Introduction

This is a desktop Matrix protocol client application that provides secure, encrypted messaging capabilities using the Matrix protocol. The application combines modern web technologies (React, TypeScript) with native desktop capabilities through Tauri and Rust.

**Purpose**: To create a lightweight, fast, and secure desktop messaging client that supports end-to-end encryption for the Matrix protocol.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────┐
│         Frontend (React/TypeScript)      │
│  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │Components│  │ Services │  │  Types │ │
│  └──────────┘  └──────────┘  └────────┘ │
└─────────────────┬───────────────────────┘
                  │ Tauri API Bridge
┌─────────────────▼───────────────────────┐
│         Backend (Rust/Tauri)            │
│  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │  Auth    │  │  Rooms   │  │Messages│ │
│  ├──────────┤  ├──────────┤  ├────────┤ │
│  │  Sync    │  │Verify    │  │ State  │ │
│  └──────────┘  └──────────┘  └────────┘ │
└─────────────────┬───────────────────────┘
                  │ Matrix SDK
┌─────────────────▼───────────────────────┐
│         Matrix Homeserver               │
└─────────────────────────────────────────┘
```

### Component Communication Flow

1. **User Interaction** → React Components
2. **Component Actions** → matrixService.ts (Service Layer)
3. **Service Calls** → Tauri Invoke API
4. **Rust Commands** → Matrix SDK Operations
5. **SDK Communication** → Matrix Homeserver
6. **Response Flow** → Reverse path back to UI

## Technology Stack

### Frontend Stack
- **React 19.1.0**: UI framework for building component-based interface
- **TypeScript 5.8**: Type-safe JavaScript for better development experience
- **Vite 7.x**: Fast build tool and development server
- **CSS**: Custom styling for modern UI design

### Backend Stack
- **Tauri 2.x**: Desktop application framework
- **Rust (Latest Stable)**: Systems programming language for backend
- **matrix-sdk**: Official Rust SDK for Matrix protocol
- **SQLite**: Local storage for session data and encryption keys
- **tokio**: Async runtime for Rust

### Build & Development Tools
- **npm**: Package manager
- **cargo**: Rust package manager
- **vite**: Frontend bundler
- **TypeScript Compiler**: Type checking and transpilation

## Project Structure

```
matrix-client/
├── docs/                          # Documentation files
├── public/                        # Static assets
├── src/                          # Frontend React application
│   ├── components/               # React components
│   │   ├── ChatView.tsx         # Message display and input
│   │   ├── Login.tsx            # Authentication UI
│   │   ├── RoomList.tsx         # Room selection list
│   │   ├── Sidebar.tsx          # Navigation sidebar
│   │   └── VerificationDialog.tsx # Device verification
│   ├── services/                # Frontend service layer
│   │   └── matrixService.ts     # API wrapper for Tauri commands
│   ├── types/                   # TypeScript type definitions
│   │   └── index.ts             # Shared type interfaces
│   ├── App.tsx                  # Main application component
│   ├── App.css                  # Application styling
│   └── main.tsx                 # React entry point
├── src-tauri/                   # Rust backend application
│   ├── src/                     # Rust source code
│   │   ├── auth.rs             # Authentication logic
│   │   ├── lib.rs              # Application setup
│   │   ├── main.rs             # Entry point
│   │   ├── messages.rs         # Message handling
│   │   ├── rooms.rs            # Room operations
│   │   ├── state.rs            # Application state
│   │   ├── sync_mod.rs         # Matrix sync operations
│   │   └── verification.rs     # Device verification
│   ├── Cargo.toml              # Rust dependencies
│   ├── tauri.conf.json         # Tauri configuration
│   └── icons/                  # Application icons
├── index.html                   # HTML entry point
├── package.json                # Node.js dependencies
├── tsconfig.json               # TypeScript configuration
└── vite.config.ts              # Vite configuration
```

## Key Features

### 1. Authentication
- Login with Matrix homeserver credentials
- Session persistence using SQLite
- Secure credential handling
- Device display name customization

### 2. Room Management
- Display all joined rooms
- Room name and topic display
- Room selection and navigation
- Real-time room list updates

### 3. Messaging
- Send and receive messages
- Message history with pagination
- Load more messages (backward pagination)
- Timestamp display
- Sender identification

### 4. End-to-End Encryption
- Device verification with emoji comparison
- Recovery key verification
- Encrypted message decryption
- Cross-signing support
- Key backup and recovery

### 5. User Interface
- Modern, clean design
- Dark theme
- Responsive layout
- Loading states and error handling
- Status notifications

## Development Workflow

### Initial Setup
```bash
# Clone repository
git clone https://github.com/thomasPeelmanUCLL/matrix-client.git
cd matrix-client

# Install frontend dependencies
npm install

# Rust and Tauri prerequisites are required (see README.md)
```

### Development Mode
```bash
# Run in development mode with hot-reload
npm run tauri dev
```
This starts:
1. Vite dev server on port 5173
2. Rust compilation
3. Tauri desktop window with dev tools

### Building for Production
```bash
# Build optimized production version
npm run tauri build
```
This creates:
1. Compiled TypeScript → JavaScript bundle
2. Optimized and minified frontend assets
3. Compiled Rust binary
4. Platform-specific installers

### Frontend Only Development
```bash
# Run just the web server
npm run dev
```
Access at `http://localhost:5173` (Tauri commands won't work)

## Security & Encryption

### Data Storage
- **Session Data**: Stored in app-specific directory using SQLite
- **Path**: `{APP_DATA_DIR}/{sanitized_user_id}/`
- **Contents**: 
  - Session tokens
  - Device keys
  - Room encryption keys
  - Cross-signing keys

### Encryption Features
1. **Device Verification**: 
   - Interactive emoji comparison
   - Recovery key backup
   - Cross-device trust establishment

2. **Message Encryption**:
   - End-to-end encryption using Olm and Megolm
   - Automatic key management
   - Key sharing between verified devices

3. **Session Security**:
   - Secure token storage
   - Automatic session cleanup on logout
   - Device-specific encryption

### Privacy Considerations
- All encryption keys stored locally
- No credentials sent to third parties
- Direct homeserver communication
- Local session management

## Configuration

### Tauri Configuration (`src-tauri/tauri.conf.json`)
- Application identifier
- Window properties
- Build settings
- Capabilities and permissions

### TypeScript Configuration (`tsconfig.json`)
- Strict type checking
- React JSX support
- ES module support

### Vite Configuration (`vite.config.ts`)
- React plugin
- Build optimizations
- Dev server settings

## Entry Points

### Frontend Entry Point
**File**: `src/main.tsx`
- Initializes React application
- Mounts to DOM element with id "root"
- Enables React Strict Mode

### Rust Entry Point
**File**: `src-tauri/src/main.rs`
- Calls the library's run function

### Library Setup
**File**: `src-tauri/src/lib.rs`
- Configures Tauri builder
- Sets up application state
- Registers command handlers
- Initializes data directory

## State Management

### Frontend State (React)
- Component-level state using `useState`
- Props for parent-child communication
- Effect hooks for side effects

### Backend State (Rust)
- `MatrixState` struct managed by Tauri
- Shared state using `Arc<RwLock<T>>`
- Thread-safe access to:
  - Matrix client instance
  - User ID
  - Verification flow ID
  - Data directory path

## Error Handling

### Frontend
- Try-catch blocks for async operations
- Error state in components
- User-friendly error messages
- Status updates during operations

### Backend
- Result<T, String> return types
- Descriptive error messages
- Logging for debugging
- Graceful error propagation

## Future Enhancement Possibilities

1. **Additional Features**
   - Media message support (images, files)
   - Voice/video calls
   - Read receipts
   - Typing indicators
   - Push notifications

2. **UI Improvements**
   - Themes (light/dark toggle)
   - User settings panel
   - Search functionality
   - Message reactions

3. **Performance**
   - Virtual scrolling for large message lists
   - Background sync
   - Offline mode support

4. **Security**
   - Biometric authentication
   - Additional verification methods
   - Security audit logging
