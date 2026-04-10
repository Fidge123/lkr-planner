# AI Agent Development Guidelines

## Project Overview

This is a desktop application which provides a planning view and integrates with [Daylite](https://developer.daylite.app/reference/getting-started), [Planradar](https://help.planradar.com/hc/en-gb/articles/15480453097373-Open-APIs) and CalDAV.

## Technical Stack

### Core Technologies

- **Framework**: Tauri v2 (Rust backend, web frontend)
- **Frontend**: React 19 + TypeScript 6
- **Runtime**: Bun (for development and dependency management)
- **Styling**: Tailwind CSS v4 + [DaisyUI](https://daisyui.com/llms.txt) components
- **Icons**: Use Lucide icons

### Development Tools

- **Linting & Formatting**: `bun lint` and `bun format` (uses Biome)
- **Testing**: `bun test` (TS unit tests), `bun test:docs` (docs check), `cargo test` (Rust unit tests and VCR tests)
- **Package Manager**: Bun (uses bun.lock)

## Code Style & Conventions

### Conventions

- All display text in the application must be German, Code and development documentation must be English
- Display user-friendly error messages in German
- Use red/green TDD
- YAGNI (You Ain't Gonna Need It): Avoid code that is not required for the current scope
- Naming:
  - **Files**: kebab-case
  - **Components**: PascalCase function components
  - **Constants**: camelCase

### Frontend

- Avoid nested `div` and `span` elements
- Create the simplest valid and semantic HTML possible

```tsx
export function ComponentName({ prop1, prop2 }: Props) {
  // Component logic
}

interface Props {
  prop1: string;
  prop2: (value: string) => void;
}
```

### API Calls

- All third-party API logic must be implemented in the Rust backend
- The frontend should only communicate with the backend via Tauri commands (`invoke`)
- Handle errors with try/catch or `.catch()` promise chains
- Return specific typed responses
- Include proper error messages with status codes

## Working with the backlog

This project uses [OpenSpec](https://github.com/Fission-AI/OpenSpec/).
Use the openspec CLI with `bunx openspec` (if that fails, use `bunx --package @fission-ai/openspec openspec`).

Document new architecture decisions as ADRs in `docs/adr`.
Whenever files in `docs/adr/` are modified, always run: `bun run test:docs`
