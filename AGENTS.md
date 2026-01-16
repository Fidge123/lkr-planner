# AI Agent Development Guidelines

## Project Overview

This is a **Tauri + React + TypeScript** desktop application which provides a planning view and integrates with [Daylite](https://developer.daylite.app/reference/getting-started) and [Planradar](https://help.planradar.com/hc/en-gb/articles/15480453097373-Open-APIs).
The application is built for macOS using Bun as the runtime and package manager.

## Technical Stack

### Core Technologies

- **Framework**: Tauri v2 (Rust backend, web frontend)
- **Frontend**: React 19 + TypeScript 5.9
- **Bundler**: Vite 7 with React plugin
- **Runtime**: Bun (for development and dependency management)
- **Styling**: Tailwind CSS v4 + DaisyUI components
- **State Management**: React useState/useEffect hooks (no external state library)

### Development Tools

- **Linting & Formatting**: Biome (all-in-one toolchain for linting and formatting)
- **Testing**: Bun test for unit testing
- **Package Manager**: Bun (uses bun.lockb)

## Code Style & Conventions

### Naming Conventions

- **Files**: camelCase for TypeScript files, PascalCase for React components
- **Components**: PascalCase function components (exported as named exports)
- **Types/Interfaces**: PascalCase with descriptive names
- **Variables**: camelCase with descriptive names
- **Constants**: camelCase (not SCREAMING_SNAKE_CASE)

### Code Patterns

#### React Components

```tsx
// Preferred pattern: Named function exports with Props interface
export function ComponentName({ prop1, prop2 }: Props) {
  // Component logic
}

interface Props {
  prop1: string;
  prop2: (value: string) => void;
}
```

#### API Calls

- Use Tauri's `@tauri-apps/plugin-http` for HTTP requests
- Handle errors with try/catch or `.catch()` Promise chains
- Return specific typed responses
- Include proper error messages with status codes

### Testing Approach

- Unit tests using Bun's built-in test runner
- Mock external APIs when needed
- Test file naming: `*.spec.ts`

### UI/UX Patterns

- **Language**: All display text in the application must be in German
- **Components**: Use DaisyUI if possible, otherwise Tailwind
- **Styling**: Utility-first Tailwind classes, avoid custom CSS
- **Interactions**: Hover states, transitions, focus management

### Error Handling

- Display user-friendly error messages in German (UI language)
- Include technical details in development/debugging modes
- Use error boundaries for React error handling
- Log errors to console for debugging

### Dependencies Philosophy

- Prefer native web APIs over external libraries when possible
- Use well-maintained, popular libraries for complex functionality
- Keep bundle size minimal - avoid heavy dependencies
- Regular updates to stay current with ecosystem

## Development Workflow

1. Use `bun tauri dev` for development server
2. To bundle the macOS app use `bun build:macos`
3. `bun lint` and `bun format` for code quality
4. Test with `bun test` before committing
