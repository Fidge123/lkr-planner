import { commands } from "../generated/tauri";

export const DAYLITE_PERSONAL_TOKEN_URL =
  "https://www.marketcircle.com/account/oauth/authorize?client_id=com.marketcircle.sample&redirect_uri=https://api.marketcircle.net/v1/personal_token/auth_code&response_type=code";
export const DEFAULT_DAYLITE_BASE_URL = "https://api.marketcircle.net/v1";

interface DayliteCommandError {
  userMessage?: string;
  user_message?: string;
}

interface LocalStoreData {
  apiEndpoints?: {
    dayliteBaseUrl?: string | null;
  };
}

interface DayliteCommandBindings {
  loadLocalStore: () => Promise<
    | { status: "ok"; data: LocalStoreData }
    | { status: "error"; error: { userMessage: string } | string }
  >;
  dayliteConnectRefreshToken: (request: {
    baseUrl: string;
    refreshToken: string;
  }) => Promise<
    | {
        status: "ok";
        data: { hasAccessToken: boolean; hasRefreshToken: boolean };
      }
    | { status: "error"; error: DayliteCommandError | string }
  >;
}

export async function resolveDayliteBaseUrl(): Promise<string> {
  const dayliteCommands = commands as unknown as DayliteCommandBindings;
  if (typeof dayliteCommands.loadLocalStore !== "function") {
    return DEFAULT_DAYLITE_BASE_URL;
  }

  try {
    const result = await dayliteCommands.loadLocalStore();
    if (result.status === "error") {
      return DEFAULT_DAYLITE_BASE_URL;
    }

    return normalizeBaseUrl(result.data.apiEndpoints?.dayliteBaseUrl);
  } catch {
    return DEFAULT_DAYLITE_BASE_URL;
  }
}

export async function updateDayliteRefreshToken(
  refreshToken: string,
): Promise<void> {
  const normalizedRefreshToken = normalizeOptionalString(refreshToken);
  if (!normalizedRefreshToken) {
    throw new Error("Bitte ein Refresh-Token eingeben.");
  }

  const dayliteCommands = commands as unknown as DayliteCommandBindings;
  if (typeof dayliteCommands.dayliteConnectRefreshToken !== "function") {
    throw new Error(
      "Die Daylite-Verbindungsfunktion ist nicht verfügbar. Bitte Anwendung neu starten.",
    );
  }

  const baseUrl = await resolveDayliteBaseUrl();
  const result = await dayliteCommands.dayliteConnectRefreshToken({
    baseUrl,
    refreshToken: normalizedRefreshToken,
  });

  if (result.status === "error") {
    throw new Error(readDayliteCommandErrorMessage(result.error));
  }
}

function normalizeBaseUrl(baseUrl: string | null | undefined): string {
  const normalizedBaseUrl = normalizeOptionalString(baseUrl);
  if (!normalizedBaseUrl) {
    return DEFAULT_DAYLITE_BASE_URL;
  }

  return normalizedBaseUrl.replace(/\/+$/, "");
}

function normalizeOptionalString(
  value: string | null | undefined,
): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }

  const normalized = value.trim();
  return normalized.length > 0 ? normalized : undefined;
}

function readDayliteCommandErrorMessage(error: DayliteCommandError | string) {
  if (typeof error === "string") {
    return error;
  }

  return (
    error.userMessage ??
    error.user_message ??
    "Das Daylite-Refresh-Token konnte nicht gespeichert werden."
  );
}
