import { describe, expect, it } from "bun:test";
import { appError } from "../test/app-error";
import { callCommand } from "./command-result";

describe("callCommand", () => {
  it("resolves the data of an ok result", async () => {
    expect(
      await callCommand(
        Promise.resolve({ status: "ok", data: "wert" }),
        "fallback",
      ),
    ).toBe("wert");
  });

  it("throws the user message of an error result", async () => {
    await expect(
      callCommand(
        Promise.resolve({ status: "error", error: appError("actual error") }),
        "fallback",
      ),
    ).rejects.toThrow("actual error");
  });

  it("throws the fallback message for an empty user message", async () => {
    await expect(
      callCommand(
        Promise.resolve({ status: "error", error: appError("") }),
        "fallback",
      ),
    ).rejects.toThrow("fallback");
  });

  it("throws the fallback message when the command rejects", async () => {
    await expect(
      callCommand(Promise.reject(new Error("IPC failure")), "fallback"),
    ).rejects.toThrow("fallback");
  });
});
