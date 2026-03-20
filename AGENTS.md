# AI Agent Development Guidelines

## Project Overview

This is a **Tauri + React + TypeScript** desktop application which provides a planning view and integrates with [Daylite](https://developer.daylite.app/reference/getting-started) and [Planradar](https://help.planradar.com/hc/en-gb/articles/15480453097373-Open-APIs).
The application is built for macOS using Bun as the runtime and package manager.

When working on this application, always follow the red-green-refactor TDD loop:
1. Write a test which fails
2. Implement the minimum amount of code to make the test pass
3. Refactor the code if needed

Before introducing a new dependency, always confirm with the user by providing a list of options with pros and cons.

## Technical Stack

### Core Technologies

- **Framework**: Tauri v2 (Rust backend, web frontend)
- **Frontend**: React 19 + TypeScript 5.8
- **Runtime**: Bun (for development and dependency management)
- **Styling**: Tailwind CSS v4 + [DaisyUI](https://daisyui.com/llms.txt) components
- **Icons**: Use Lucide icons

### Development Tools

- **Linting & Formatting**: Biome (all-in-one toolchain for linting and formatting)
- **Testing**: Bun test for unit testing
- **Package Manager**: Bun (uses bun.lockb)

## Code Style & Conventions

### Conventions

- **Files**: kebab-case
- **Components**: PascalCase function components
- **Constants**: camelCase
- All display text in the application must be German, Code and development documentation must be English
- Display user-friendly error messages in German (UI language)
- Create the simplest valid and semantic HTML possible
- Avoid nested `div` and `span` elements
- YAGNI (You Ain't Gonna Need It): Avoid code that is not required for the current scope

### React Components

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

### Testing Approach

- Unit tests using Bun's built-in test runner
- Mock external APIs when needed
- Test file naming: `*.spec.ts`

## Development Workflow

Validate results after every tasks:

1. `bun lint` for code quality and formatting
2. `cargo check` in `src-tauri/` to test Rust correctness
3. `bun build` to test Typescript correctness
4. Test typescript code with `bun test`
5. Test rust code with `cargo test` in `src-tauri/`

## Working with the backlog

This project uses openspec.
Follow Red-Green-Refactor TDD loop.
Document new architecture decisions as ADRs in `docs/adr`.
ADRs need to include sections for Context (including evaluated options with pros and cons), Decision, and Consequences.

Whenever files in `docs/adr/` are modified, always run:
- `bun run test:docs`
This check is mandatory for every ADR change.
