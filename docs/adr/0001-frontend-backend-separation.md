# ADR 0001: Frontend/Backend Separation in Tauri

- Status: Accepted
- Date: 2026-02-13

## Context

LKR Planner is built with React/TypeScript in the frontend and Rust in the Tauri backend.
The project needs a clear boundary between UI concerns and integration/system concerns to keep the codebase maintainable and testable.

## Decision

The frontend is responsible for UI rendering, user interaction, and local presentation state.
The Tauri backend is responsible for network calls, filesystem access, secret handling, and performance-critical integration logic.

## Consequences

- UI code remains focused on interaction and presentation concerns.
- External integrations and sensitive operations stay in Rust commands.
- The separation enables clearer test boundaries and lower coupling across layers.
