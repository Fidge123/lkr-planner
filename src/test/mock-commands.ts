import { afterAll, mock } from "bun:test";
import { commands } from "../generated/tauri";

const realCommands = commands;

/** Keys are checked against the real commands; shapes are not, so a spec can stub a narrowed result. */
type CommandOverrides = { [K in keyof typeof commands]?: unknown };

/**
 * `mock.module` is process-wide, so the real commands return after the calling spec file.
 * Overrides land on top of the real commands, which fail loudly when invoked.
 */
export function mockCommands(overrides: CommandOverrides): void {
  mock.module("../generated/tauri", () => ({
    commands: { ...realCommands, ...overrides },
  }));
  afterAll(() => {
    mock.module("../generated/tauri", () => ({ commands: realCommands }));
  });
}
