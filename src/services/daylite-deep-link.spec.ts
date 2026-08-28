import { describe, expect, it, mock } from "bun:test";
import { mockCommands } from "../test/mock-commands";

type OpenProjectResult =
  | { status: "ok"; data: null }
  | { status: "error"; error: string };

const mockOpenProject = mock(
  (_projectRef: string): Promise<OpenProjectResult> =>
    Promise.resolve({ status: "ok", data: null }),
);

mockCommands({ dayliteOpenProject: mockOpenProject });

const { openProjectInDaylite } = await import("./daylite-deep-link");

describe("openProjectInDaylite", () => {
  it("hands the project reference to the backend", async () => {
    mockOpenProject.mockClear();

    await openProjectInDaylite("/v1/projects/2035");

    expect(mockOpenProject).toHaveBeenCalledWith("/v1/projects/2035");
  });

  it("returns the German message of a failing command instead of throwing", async () => {
    mockOpenProject.mockImplementationOnce(() =>
      Promise.resolve({
        status: "error",
        error: "Das Projekt konnte in Daylite nicht geöffnet werden.",
      }),
    );

    expect(await openProjectInDaylite("/v1/projects/2035")).toBe(
      "Das Projekt konnte in Daylite nicht geöffnet werden.",
    );
  });

  it("returns a German message when the command itself fails", async () => {
    mockOpenProject.mockImplementationOnce(() =>
      Promise.reject(new Error("ipc down")),
    );

    expect(await openProjectInDaylite("/v1/projects/2035")).toBe(
      "Das Projekt konnte in Daylite nicht geöffnet werden.",
    );
  });

  it("returns null when the project was opened", async () => {
    expect(await openProjectInDaylite("/v1/projects/2035")).toBeNull();
  });
});
