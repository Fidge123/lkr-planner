import { beforeEach, describe, expect, it, mock } from "bun:test";
import type { LocalStore } from "../generated/tauri";
import { mockCommands } from "../test/mock-commands";
import { dayliteBaseUrlOf, updateDayliteRefreshToken } from "./daylite-auth";

const mockDayliteConnectRefreshToken = mock(() =>
  Promise.resolve({} as unknown),
);

mockCommands({
  dayliteConnectRefreshToken: mockDayliteConnectRefreshToken,
});

describe("daylite auth service", () => {
  beforeEach(() => {
    mockDayliteConnectRefreshToken.mockClear();
  });

  it("falls back to the default base url when none is stored", () => {
    const store = { apiEndpoints: { dayliteBaseUrl: "  " } } as LocalStore;

    expect(dayliteBaseUrlOf(store)).toBe("https://api.marketcircle.net/v1");
  });

  it("updates refresh token with resolved base url", async () => {
    mockDayliteConnectRefreshToken.mockResolvedValue({
      status: "ok",
      data: {
        hasAccessToken: true,
        hasRefreshToken: true,
      },
    });

    await updateDayliteRefreshToken({
      baseUrl: " https://api.marketcircle.net/v1/ ",
      refreshToken: " refresh-token-123 ",
    });

    expect(mockDayliteConnectRefreshToken).toHaveBeenCalledWith({
      baseUrl: "https://api.marketcircle.net/v1",
      refreshToken: "refresh-token-123",
    });
  });

  it("throws a german message when refresh token is empty", async () => {
    await expect(
      updateDayliteRefreshToken({
        baseUrl: "https://api.marketcircle.net/v1",
        refreshToken: "   ",
      }),
    ).rejects.toThrow("Bitte ein Refresh-Token eingeben.");
  });

  it("throws a german message when daylite base url is empty", async () => {
    await expect(
      updateDayliteRefreshToken({
        baseUrl: "   ",
        refreshToken: "refresh-token-123",
      }),
    ).rejects.toThrow("Bitte eine Daylite-API-URL eingeben.");
  });
});
