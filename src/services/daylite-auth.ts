import { commands, type DayliteRefreshTokenRequest } from "../generated/tauri";
import {
  normalizeOptionalString,
  readDayliteApiErrorMessage,
} from "./daylite-service-helpers";

export const DAYLITE_PERSONAL_TOKEN_URL =
  "https://www.marketcircle.com/account/oauth/authorize?client_id=com.marketcircle.sample&redirect_uri=https://api.marketcircle.net/v1/personal_token/auth_code&response_type=code";
export const DEFAULT_DAYLITE_BASE_URL = "https://api.marketcircle.net/v1";

export async function resolveDayliteBaseUrl(): Promise<string> {
  try {
    const result = await commands.loadLocalStore();
    if (result.status === "error") {
      return DEFAULT_DAYLITE_BASE_URL;
    }

    return normalizeBaseUrl(result.data.apiEndpoints?.dayliteBaseUrl);
  } catch {
    return DEFAULT_DAYLITE_BASE_URL;
  }
}

export async function updateDayliteRefreshToken(
  input: DayliteRefreshTokenRequest,
): Promise<void> {
  const normalizedBaseUrl = normalizeOptionalString(input.baseUrl)?.replace(
    /\/+$/,
    "",
  );
  if (!normalizedBaseUrl) {
    throw new Error("Bitte eine Daylite-API-URL eingeben.");
  }

  const normalizedRefreshToken = normalizeOptionalString(input.refreshToken);
  if (!normalizedRefreshToken) {
    throw new Error("Bitte ein Refresh-Token eingeben.");
  }

  const result = await commands.dayliteConnectRefreshToken({
    baseUrl: normalizedBaseUrl,
    refreshToken: normalizedRefreshToken,
  });

  if (result.status === "error") {
    throw new Error(
      readDayliteApiErrorMessage(
        result.error,
        "Das Daylite-Refresh-Token konnte nicht gespeichert werden.",
      ),
    );
  }
}

function normalizeBaseUrl(baseUrl: string | null | undefined): string {
  const normalizedBaseUrl = normalizeOptionalString(baseUrl);
  if (!normalizedBaseUrl) {
    return DEFAULT_DAYLITE_BASE_URL;
  }

  return normalizedBaseUrl.replace(/\/+$/, "");
}
