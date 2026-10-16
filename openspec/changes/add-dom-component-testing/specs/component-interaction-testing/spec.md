## Purpose

Lets component tests exercise a component the way a user does, so behavior that only appears after an effect, a state change or an event can be asserted instead of assumed.

## ADDED Requirements

### Requirement: Components run in a DOM during tests
The system SHALL provide a browser-like document to `bun test`, so a mounted component runs its effects, updates on state changes and responds to dispatched events.

#### Scenario: Effects run on mount
- **WHEN** a test mounts a component whose effect sets state
- **THEN** the rendered output reflects the state the effect set

#### Scenario: A user event updates the rendered output
- **WHEN** a test dispatches a click on a rendered control that toggles state
- **THEN** the rendered output reflects the new state

#### Scenario: Pure logic tests are unaffected
- **WHEN** the suite runs a spec file that renders nothing
- **THEN** it passes without a DOM and without extra setup

### Requirement: Tests query the accessible tree
The system SHALL let tests find rendered elements by their role, label or text, so an assertion states what the user can see and reach rather than how the markup happens to be shaped.

#### Scenario: Finding a control by what it is
- **WHEN** a test asserts on a control the user would identify by its label
- **THEN** it locates that control without matching raw HTML

#### Scenario: A disabled control is reported as disabled
- **WHEN** a test asserts that a control the user can see is not usable
- **THEN** the assertion reflects the control's disabled state directly

### Requirement: Component state is reachable by interaction
The system SHALL let a test reach any state a user can reach by performing the interactions that lead there, so a component needs no test-only entry points into its own states.

#### Scenario: Reaching a confirmation state
- **WHEN** a test needs a component in a state the user reaches by clicking
- **THEN** it clicks that control rather than passing the state in as a prop

#### Scenario: State resets between sessions
- **WHEN** a test closes and reopens a component that resets state on open
- **THEN** the reopened component shows the reset state

#### Scenario: No test-only props remain
- **WHEN** a component is inspected for props that only tests pass
- **THEN** none are present

### Requirement: The suite stays a single command
The system SHALL keep every frontend test runnable with the existing `bun test`, with no separate runner, no running application and no network.

#### Scenario: One command runs everything
- **WHEN** the developer runs `bun test`
- **THEN** DOM component tests and pure logic tests run together in one pass

#### Scenario: Tests leave no shared state behind
- **WHEN** two tests mount the same component in one run
- **THEN** neither observes the document or state left by the other
