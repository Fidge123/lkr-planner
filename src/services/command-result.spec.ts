import { describe, expect, it } from "bun:test";
import { callCommand, unwrapCommandResult } from "./command-result";

describe("unwrapCommandResult", () => {
  it("returns the data on ok results", () => {
    const value = unwrapCommandResult(
      { status: "ok", data: "wert" },
      "fallback",
    );

    expect(value).toBe("wert");
  });

  it("throws plain string errors unchanged", () => {
    expect(() =>
      unwrapCommandResult(
        { status: "error", error: "actual error" },
        "fallback",
      ),
    ).toThrow("actual error");
  });

  it("throws the userMessage from api error objects when available", () => {
    expect(() =>
      unwrapCommandResult(
        {
          status: "error",
          error: {
            userMessage: "user error",
            code: "UNAUTHORIZED",
            httpStatus: 401,
            technicalMessage: "tech msg",
          },
        },
        "fallback",
      ),
    ).toThrow("user error");
  });

  it("throws the fallback message for an empty error string", () => {
    expect(() =>
      unwrapCommandResult({ status: "error", error: "" }, "fallback"),
    ).toThrow("fallback");
  });

  it("throws the fallback message when the error has no userMessage", () => {
    expect(() =>
      unwrapCommandResult(
        { status: "error", error: { code: "UNAUTHORIZED" } },
        "fallback",
      ),
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

  it("throws the error of an error result", async () => {
    await expect(
      callCommand(
        Promise.resolve({ status: "error", error: "actual error" }),
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
