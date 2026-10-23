# AI Agent Development Guidelines

## Project Overview

This is a desktop application which provides a planning view and integrates with [Daylite](https://developer.daylite.app/reference/getting-started), [Planradar](https://help.planradar.com/hc/en-gb/articles/15480453097373-Open-APIs) and CalDAV.

## Code Style & Conventions

### Conventions

- All display text in the application must be German, Code and development documentation must be English
- Use Lucide icons and [DaisyUI](https://daisyui.com/llms.txt) components
- Display user-friendly error messages in German
- Use red/green TDD, but be sensible what tests are actually needed
- YAGNI (You Ain't Gonna Need It): Avoid code that is not required for the current scope
- Comments are a code smell
  - Only write a comment when it prevents future errors a skilled developer would be surprised by
  - Never narrate or restate the code, never reference previous changes or project history
  - Keep comments terse, ideally one short line; state the fact in fewer words rather than wrapping it
  - Break a comment across lines only when it carries more than one fact, and break at sentence ends or semantic boundaries, never mid-sentence
  - Proactive improve or remove comments that do not follow these conventions
- Naming:
  - **Files**: kebab-case
  - **Components**: PascalCase function components
  - **Constants**: camelCase, which Biome enforces for module-level constants

### Documentation

All markdown files and pull request descriptions should have one sentence per line.
Don't break sentences across multiple lines in markdown.
Avoid `---` between headers and avoid em dashes.

### Frontend

- Avoid nested `div` and `span` elements
- Destructure props in the signature and declare the `Props` interface below the component

## Architecture

### Backend (`src-tauri/src`)

- All third-party API logic is implemented in the Rust backend
- Commands wrap their body in `observe` for telemetry and return `Result<T, AppError>` when they can fail
- Read the local store with `load_local_store` and change it only through `update_local_store`
- Send HTTP requests through `crate::http::shared_client`, and Clippy rejects building another client
- A process-wide singleton is a `static OnceLock` in the module that owns it; Tauri managed state is only for state that needs the app handle at startup, like telemetry

### Frontend (`src`)

- Only modules in `src/services` call the generated `commands`, each through `callCommand`, and Biome rejects the import anywhere else
- Components read backend data through query hooks in `src/app/hooks` and edit displayed weeks through `useWeekCache`
- A component that only renders receives its data as props, so its specs need no seeded queries
- Component specs mount through `renderComponent` and reach a state by clicking or typing, never through props that only tests set
- The React Compiler memoizes components, so add `useMemo` or `useCallback` only where a measurement shows it misses a value
- Read the date through `useToday` and pass `today` to date helpers, never read the clock during render

### Adding a feature

1. Add the Rust command with its unit tests; `cargo test regenerate_generated_bindings` updates `src/generated/tauri.ts`
2. Add a service function in `src/services` that calls the command through `callCommand`
3. Read through a query hook, or write through the service and update or invalidate the affected queries
4. Render with DaisyUI components and German text

Aim for at most 500 added lines per pull request, tests included, and at most 100 lines of OpenSpec markdown per change; split larger work into steps.

## Working with the backlog

This project uses [OpenSpec](https://github.com/Fission-AI/OpenSpec/).
Use the openspec CLI with `bunx openspec` (if that fails, use `bunx --package @fission-ai/openspec openspec`).

Record a decision as an ADR in `docs/adr` only when it spans capabilities or sets a convention; a feature's own decisions belong in its `design.md`.
