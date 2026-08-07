# AI Agent Development Guidelines

## Project Overview

This is a desktop application which provides a planning view and integrates with [Daylite](https://developer.daylite.app/reference/getting-started), [Planradar](https://help.planradar.com/hc/en-gb/articles/15480453097373-Open-APIs) and CalDAV.

## Code Style & Conventions

### Conventions

- All display text in the application must be German, Code and development documentation must be English
- Use Lucide icons
- Use [DaisyUI](https://daisyui.com/llms.txt) components
- Display user-friendly error messages in German
- Use red/green TDD
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
  - **Constants**: camelCase

### Documentation

All markdown files and pull request descriptions should have one sentence per line.
Don't break sentences across multiple lines in markdown.
Avoid `---` between headers and avoid em dashes.

### Frontend

- Avoid nested `div` and `span` elements
- Destructure props in the signature and declare the `Props` interface below the component

### API Calls

- All third-party API logic must be implemented in the Rust backend
- The frontend should only communicate with the backend via Tauri commands (`invoke`)

## Working with the backlog

This project uses [OpenSpec](https://github.com/Fission-AI/OpenSpec/).
Use the openspec CLI with `bunx openspec` (if that fails, use `bunx --package @fission-ai/openspec openspec`).

Document new architecture decisions as ADRs in `docs/adr`.
