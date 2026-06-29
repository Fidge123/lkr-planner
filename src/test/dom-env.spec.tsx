import { describe, expect, test } from "bun:test";
import { render, screen } from "@testing-library/react";

// Proves the happy-dom + React Testing Library environment is wired up, so the
// fixture-driven component tests have a DOM to render into under `bun test`.
describe("DOM test environment", () => {
  test("renders a React element into the document", () => {
    render(<p>ready</p>);

    expect(screen.getByText("ready").textContent).toBe("ready");
  });
});
