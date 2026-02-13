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
- **Styling**: Tailwind CSS v4 + DaisyUI components

### Development Tools

- **Linting & Formatting**: Biome (all-in-one toolchain for linting and formatting)
- **Testing**: Bun test for unit testing
- **Package Manager**: Bun (uses bun.lockb)

## Code Style & Conventions

### Naming Conventions

- **Files**: kebab-case
- **Components**: PascalCase function components
- **Constants**: camelCase

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

### Semantic Markup

- Create the simplest valid and semantic HTML possible
- Avoid nested `div` and `span` elements

### API Calls

- Use Tauri's `@tauri-apps/plugin-http` for HTTP requests
- Handle errors with try/catch or `.catch()` promise chains
- Return specific typed responses
- Include proper error messages with status codes

### Testing Approach

- Unit tests using Bun's built-in test runner
- Mock external APIs when needed
- Test file naming: `*.spec.ts`

### UI/UX Patterns

- **Language**: All display text in the application must be German, Code and development documentation must be English
- **Components**: Use DaisyUI if possible, otherwise Tailwind
- **Styling**: Utility-first Tailwind classes, avoid custom CSS
- **Icons**: Use Lucide icons

### Error Handling

- Display user-friendly error messages in German (UI language)
- Include technical details in development/debugging modes
- Use error boundaries for React error handling
- Log errors to console for debugging

## Development Workflow

1. Use `bun tauri dev` for development server
2. To bundle the macOS app use `bun build:macos`
3. `bun lint` and `bun format` for code quality
4. Test with `bun test` before committing

## Working with the backlog

Unless otherwise instructed, always work on the highest priority backlog item from `docs/BACKLOG.md`.
Verify first, if the backlog item contains all information that you need to implement it.
Check if the acceptance criteria are clear and testable.
Follow Red-Green-Refactor TDD loop.
Document new architecture decisions as ADRs in `docs/adr`.
If you have questions or need clarification, ask the user.
Once you are done, update the backlog item and move it to the `docs/COMPLETED_BACKLOG.md`.
If there are follow-up tasks necessary, add them to the backlog.
