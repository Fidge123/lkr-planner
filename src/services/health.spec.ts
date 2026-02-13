import { beforeEach, describe, expect, it, mock } from "bun:test";
import { checkHealth } from "./health";

// Mock the Tauri invoke function
const mockInvoke = mock(() => Promise.resolve({} as unknown));

// Mock @tauri-apps/api/core module
mock.module("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

describe("health service", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("checkHealth", () => {
    it("should call the check_health command", async () => {
      const mockResponse = {
        status: "healthy" as const,
        timestamp: "2026-02-13T10:00:00Z",
        version: "0.1.0",
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await checkHealth();

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith("check_health");
      expect(result).toEqual(mockResponse);
    });

    it("should return healthy status with timestamp and version", async () => {
      const mockResponse = {
        status: "healthy" as const,
        timestamp: "2026-02-13T10:00:00Z",
        version: "0.1.0",
      };

      mockInvoke.mockResolvedValue(mockResponse);

      const result = await checkHealth();

      expect(result.status).toBe("healthy");
      expect(result.timestamp).toBeTruthy();
      expect(result.version).toBeTruthy();
    });

    it("should throw a German error message when the command fails", async () => {
      mockInvoke.mockRejectedValue(new Error("Backend nicht erreichbar"));

      await expect(checkHealth()).rejects.toThrow(
        "Health check fehlgeschlagen: Backend nicht erreichbar",
      );
    });

    it("should handle non-Error rejections", async () => {
      mockInvoke.mockRejectedValue("String error");

      await expect(checkHealth()).rejects.toThrow(
        "Health check fehlgeschlagen: String error",
      );
    });
  });
});
