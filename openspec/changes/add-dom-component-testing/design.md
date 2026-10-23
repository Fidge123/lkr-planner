## Context

Fourteen spec files render components statically, mostly asserting with `toContain` on the HTML string; the other specs test pure functions.
Components that read queries render inside a `QueryClientProvider` through `renderWithQueries`.

## Decisions

### happy-dom registered once through a `bunfig.toml` preload
This is the setup Bun documents for DOM tests, and the preload keeps it out of every spec.
jsdom is more complete but slower to start, and the components under test are dialogs, lists and a grid.

### Testing Library mounts, queries and cleans up
`@testing-library/react` 16 supports React 19's `act`, and its role and label queries replace regular expressions over markup.
`fireEvent` covers the clicks and changes the specs need, so `user-event` and `jest-dom` stay out until a test needs them.

### One mounting helper replaces both static renderers
`renderComponent` mounts inside a fresh query client that can be seeded, so every component spec mounts the same way.
Render-only specs keep their assertions on the mounted markup, which keeps the conversion mechanical; interaction specs query by role and label.

## Risks / Trade-offs

- [Mounted components run their effects, so a query without seeded data calls a command] → Seed the data the spec needs, and let an unmocked command fail into the query's error state.
- [happy-dom serializes some attributes, such as inline styles, differently from the static renderer] → Assert those through the element's properties instead of the markup.
- [Every spec file pays the DOM start-up cost] → Accepted for a single runner configuration.
