# ADR 0002: Service Facade and Integration Module Structure

- Status: Accepted
- Date: 2026-02-13

## Context

Frontend components require a stable way to access backend commands without scattering raw command calls across UI code.
Integrations (Daylite, Planradar, iCal) also need a predictable location in the Rust codebase.

## Decision

Frontend components consume backend functionality only through service modules in `src/services/*.ts`.
Service modules encapsulate Tauri command invocations and normalize errors for UI usage.
Rust integration modules are organized under `src-tauri/src/integrations/`.

## Consequences

- UI code can evolve without direct dependency on command names.
- Error handling becomes consistent across frontend call sites.
- Integration code has a single, discoverable structure for future expansion.
