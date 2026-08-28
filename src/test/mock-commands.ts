import { mock } from "bun:test";
import { commands } from "../generated/tauri";

const realCommands = commands;

/** Keys are checked against the real commands; shapes are not, so a spec can stub a narrowed result. */
type CommandOverrides = { [K in keyof typeof commands]?: unknown };

/**
 * `mock.module` patches the registry for the whole test process, not just the
 * declaring file, so a partial replacement leaves later specs with a `commands`
 * object missing everything the declaring spec did not need.
 * Overrides land on top of the real commands, which fail loudly when invoked.
 */
export function mockCommands(overrides: CommandOverrides): void {
  mock.module("../generated/tauri", () => ({
    commands: { ...realCommands, ...overrides },
  }));
}
