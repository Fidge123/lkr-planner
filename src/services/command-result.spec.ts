import { describe, expect, it } from "bun:test";
import { appError } from "../test/app-error";
import { callCommand, unwrapCommandResult } from "./command-result";

describe("unwrapCommandResult", () => {
  it("returns the data on ok results", () => {
    const value = unwrapCommandResult(
      { status: "ok", data: "wert" },
      "fallback",
    );

    expect(value).toBe("wert");
  });

  it("throws the user message of the error", () => {
    expect(() =>
      unwrapCommandResult(
        { status: "error", error: appError("user error") },
        "fallback",
      ),
    ).toThrow("user error");
  });

  it("throws the fallback message for an empty user message", () => {
    expect(() =>
      unwrapCommandResult({ status: "error", error: appError("") }, "fallback"),
    ).toThrow("fallback");
  });
});

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

  it("throws the fallback message when the command rejects", async () => {
    await expect(
      callCommand(Promise.reject(new Error("IPC failure")), "fallback"),
    ).rejects.toThrow("fallback");
  });
});
