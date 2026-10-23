import { commands, type DayliteRefreshTokenRequest } from "../generated/tauri";
import { callCommand } from "./command-result";
import { normalizeOptionalString } from "./daylite-service-helpers";
import { loadLocalStore } from "./local-store";

export const daylitePersonalTokenUrl =
  "https://www.marketcircle.com/account/oauth/authorize?client_id=com.marketcircle.sample&redirect_uri=https://api.marketcircle.net/v1/personal_token/auth_code&response_type=code";
export const defaultDayliteBaseUrl = "https://api.marketcircle.net/v1";

export async function resolveDayliteBaseUrl(): Promise<string> {
  const store = await loadLocalStore().catch(() => null);
  return normalizeBaseUrl(store?.apiEndpoints?.dayliteBaseUrl);
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

  await callCommand(
    commands.dayliteConnectRefreshToken({
      baseUrl: normalizedBaseUrl,
      refreshToken: normalizedRefreshToken,
    }),
    "Das Daylite-Refresh-Token konnte nicht gespeichert werden.",
  );
}

function normalizeBaseUrl(baseUrl: string | null | undefined): string {
  const normalizedBaseUrl = normalizeOptionalString(baseUrl);
  if (!normalizedBaseUrl) {
    return defaultDayliteBaseUrl;
  }

  return normalizedBaseUrl.replace(/\/+$/, "");
}
