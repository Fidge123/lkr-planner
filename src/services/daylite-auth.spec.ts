import { beforeEach, describe, expect, it, mock } from "bun:test";
import {
  DEFAULT_DAYLITE_BASE_URL,
  resolveDayliteBaseUrl,
  updateDayliteRefreshToken,
} from "./daylite-auth";

const mockLoadLocalStore = mock(() => Promise.resolve({} as unknown));
const mockDayliteConnectRefreshToken = mock(() =>
  Promise.resolve({} as unknown),
);

mock.module("../generated/tauri", () => ({
  commands: {
    loadLocalStore: mockLoadLocalStore,
    dayliteConnectRefreshToken: mockDayliteConnectRefreshToken,
  },
}));

describe("daylite auth service", () => {
  beforeEach(() => {
    mockLoadLocalStore.mockClear();
    mockDayliteConnectRefreshToken.mockClear();
  });

  it("resolves configured daylite base url from store", async () => {
    mockLoadLocalStore.mockResolvedValue({
      status: "ok",
      data: {
        apiEndpoints: {
          dayliteBaseUrl: " https://api.marketcircle.net/v1/ ",
        },
      },
    });

    const baseUrl = await resolveDayliteBaseUrl();

    expect(baseUrl).toBe("https://api.marketcircle.net/v1");
  });

  it("falls back to default base url when store is unavailable", async () => {
    mockLoadLocalStore.mockResolvedValue({
      status: "error",
      error: { userMessage: "Lesefehler" },
    });

    const baseUrl = await resolveDayliteBaseUrl();

    expect(baseUrl).toBe(DEFAULT_DAYLITE_BASE_URL);
  });

  it("updates refresh token with resolved base url", async () => {
    mockLoadLocalStore.mockResolvedValue({
      status: "ok",
      data: {
        apiEndpoints: {
          dayliteBaseUrl: "https://api.marketcircle.net/v1",
        },
      },
    });
    mockDayliteConnectRefreshToken.mockResolvedValue({
      status: "ok",
      data: {
        hasAccessToken: true,
        hasRefreshToken: true,
      },
    });

    await updateDayliteRefreshToken(" refresh-token-123 ");

    expect(mockDayliteConnectRefreshToken).toHaveBeenCalledWith({
      baseUrl: "https://api.marketcircle.net/v1",
      refreshToken: "refresh-token-123",
    });
  });

  it("throws a german message when refresh token is empty", async () => {
    await expect(updateDayliteRefreshToken("   ")).rejects.toThrow(
      "Bitte ein Refresh-Token eingeben.",
    );
  });

  it("throws daylite command error message on connect failure", async () => {
    mockLoadLocalStore.mockResolvedValue({
      status: "ok",
      data: {
        apiEndpoints: {
          dayliteBaseUrl: "https://api.marketcircle.net/v1",
        },
      },
    });
    mockDayliteConnectRefreshToken.mockResolvedValue({
      status: "error",
      error: {
        userMessage: "Token konnte nicht validiert werden.",
      },
    });

    await expect(
      updateDayliteRefreshToken("refresh-token-123"),
    ).rejects.toThrow("Token konnte nicht validiert werden.");
  });
});
