import { ExternalLink } from "lucide-react";
import { type ChangeEvent, useEffect, useState } from "react";
import {
  DAYLITE_PERSONAL_TOKEN_URL,
  DEFAULT_DAYLITE_BASE_URL,
  resolveDayliteBaseUrl,
  updateDayliteRefreshToken,
} from "../../../services/daylite-auth";
import { type PanelStatus, StatusAlert } from "./panel-status";

export function DayliteSettingsPanel({ onClose }: Props) {
  const [dayliteBaseUrlInput, setDayliteBaseUrlInput] = useState(
    DEFAULT_DAYLITE_BASE_URL,
  );
  const [refreshTokenInput, setRefreshTokenInput] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [status, setStatus] = useState<PanelStatus | null>(null);

  useEffect(() => {
    setStatus(null);
    setRefreshTokenInput("");
    let isActive = true;
    void resolveDayliteBaseUrl().then((resolvedBaseUrl) => {
      if (!isActive) {
        return;
      }
      setDayliteBaseUrlInput(resolvedBaseUrl);
    });
    return () => {
      isActive = false;
    };
  }, []);

  const onSubmit = async (event: ChangeEvent<HTMLFormElement>) => {
    event.preventDefault();
    setIsSaving(true);
    setStatus(null);

    try {
      await updateDayliteRefreshToken({
        baseUrl: dayliteBaseUrlInput,
        refreshToken: refreshTokenInput,
      });
      setStatus({
        type: "success",
        message: "Daylite-Konfiguration wurde aktualisiert.",
      });
      setRefreshTokenInput("");
    } catch (error) {
      setStatus({
        type: "error",
        message:
          error instanceof Error
            ? error.message
            : "Das Daylite-Refresh-Token konnte nicht gespeichert werden.",
      });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <>
      <h3 className="text-lg font-semibold">Daylite-Konfiguration</h3>

      <StatusAlert status={status} />

      <form className="mt-4 flex flex-col gap-4" onSubmit={onSubmit}>
        <label className="form-control w-full">
          <span className="label-text mb-2">Daylite API-URL</span>
          <input
            type="url"
            className="input input-bordered w-full"
            value={dayliteBaseUrlInput}
            onChange={(event) => setDayliteBaseUrlInput(event.target.value)}
            disabled={isSaving}
            placeholder="https://api.marketcircle.net/v1"
          />
        </label>

        <label className="form-control w-full">
          <span className="label-text mb-2">Refresh-Token</span>
          <input
            type="password"
            className="input input-bordered w-full"
            value={refreshTokenInput}
            onChange={(event) => setRefreshTokenInput(event.target.value)}
            disabled={isSaving}
            placeholder="Refresh-Token eingeben"
          />
        </label>

        <section className="flex items-center justify-between gap-3">
          <a
            className="btn btn-ghost btn-sm"
            href={DAYLITE_PERSONAL_TOKEN_URL}
            target="_blank"
            rel="noreferrer"
          >
            <ExternalLink className="size-4" />
            Token abrufen
          </a>
          <section className="flex items-center gap-2">
            <button
              type="button"
              className="btn btn-sm"
              onClick={onClose}
              disabled={isSaving}
            >
              Schließen
            </button>
            <button
              type="submit"
              className="btn btn-primary btn-sm"
              disabled={isSaving}
            >
              {isSaving ? "Speichere..." : "Speichern"}
            </button>
          </section>
        </section>
      </form>
    </>
  );
}

interface Props {
  onClose: () => void;
}
