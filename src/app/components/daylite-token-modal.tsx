import { ExternalLink } from "lucide-react";
import { type ChangeEvent, useEffect, useState } from "react";
import {
  DAYLITE_PERSONAL_TOKEN_URL,
  DEFAULT_DAYLITE_BASE_URL,
  resolveDayliteBaseUrl,
  updateDayliteRefreshToken,
} from "../../services/daylite-auth";

export function DayliteTokenModal({ isOpen, onClose }: DayliteTokenModalProps) {
  const [dayliteBaseUrlInput, setDayliteBaseUrlInput] = useState(
    DEFAULT_DAYLITE_BASE_URL,
  );
  const [refreshTokenInput, setRefreshTokenInput] = useState("");
  const [isSavingRefreshToken, setIsSavingRefreshToken] = useState(false);
  const [refreshTokenStatus, setRefreshTokenStatus] =
    useState<RefreshTokenStatus | null>(null);

  const requestClose = () => {
    if (isSavingRefreshToken) {
      return;
    }

    onClose();
  };
  const onRefreshTokenSubmit = async (event: ChangeEvent<HTMLFormElement>) => {
    event.preventDefault();
    setIsSavingRefreshToken(true);
    setRefreshTokenStatus(null);

    try {
      await updateDayliteRefreshToken({
        baseUrl: dayliteBaseUrlInput,
        refreshToken: refreshTokenInput,
      });
      setRefreshTokenStatus({
        type: "success",
        message: "Daylite-Konfiguration wurde aktualisiert.",
      });
      setRefreshTokenInput("");
    } catch (error) {
      setRefreshTokenStatus({
        type: "error",
        message:
          error instanceof Error
            ? error.message
            : "Das Daylite-Refresh-Token konnte nicht gespeichert werden.",
      });
    } finally {
      setIsSavingRefreshToken(false);
    }
  };

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    setRefreshTokenStatus(null);
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
  }, [isOpen]);

  if (!isOpen) {
    return null;
  }

  return (
    <dialog
      className="modal modal-open"
      open
      aria-labelledby="daylite-token-modal-title"
    >
      <section className="modal-box max-w-xl">
        <h2 id="daylite-token-modal-title" className="text-lg font-semibold">
          Daylite-Konfiguration
        </h2>

        {refreshTokenStatus ? (
          <section
            className={`alert mt-4 ${
              refreshTokenStatus.type === "success"
                ? "alert-success"
                : "alert-error"
            }`}
          >
            <span>{refreshTokenStatus.message}</span>
          </section>
        ) : null}

        <form
          className="mt-4 flex flex-col gap-4"
          onSubmit={onRefreshTokenSubmit}
        >
          <label className="form-control w-full">
            <span className="label-text mb-2">Daylite API-URL</span>
            <input
              type="url"
              className="input input-bordered w-full"
              value={dayliteBaseUrlInput}
              onChange={(event) => setDayliteBaseUrlInput(event.target.value)}
              disabled={isSavingRefreshToken}
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
              disabled={isSavingRefreshToken}
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
                onClick={requestClose}
                disabled={isSavingRefreshToken}
              >
                Schließen
              </button>
              <button
                type="submit"
                className="btn btn-primary btn-sm"
                disabled={isSavingRefreshToken}
              >
                {isSavingRefreshToken ? "Speichere..." : "Speichern"}
              </button>
            </section>
          </section>
        </form>
      </section>
      <button
        type="button"
        className="modal-backdrop"
        onClick={requestClose}
        aria-label="Dialog schließen"
      >
        Schließen
      </button>
    </dialog>
  );
}

interface DayliteTokenModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface RefreshTokenStatus {
  type: "success" | "error";
  message: string;
}
