import { describe, expect, it } from "bun:test";
import { unwrapCommandResult } from "./command-result";

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

  it("throws the fallback message when the error has no userMessage", () => {
    expect(() =>
      unwrapCommandResult(
        { status: "error", error: { code: "UNAUTHORIZED" } },
        "fallback",
      ),
    ).toThrow("fallback");
  });
});
