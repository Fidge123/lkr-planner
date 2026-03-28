import { ExternalLink } from "lucide-react";
import { type ChangeEvent, useEffect, useState } from "react";
import type { ZepCredentialsInfo } from "../../generated/tauri";
import {
  DAYLITE_PERSONAL_TOKEN_URL,
  DEFAULT_DAYLITE_BASE_URL,
  resolveDayliteBaseUrl,
  updateDayliteRefreshToken,
} from "../../services/daylite-auth";
import {
  loadZepCredentials,
  saveZepCredentials,
  testZepCredentials,
} from "../../services/zep";

type SettingsSection = "daylite" | "zep";

export function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const [activeSection, setActiveSection] =
    useState<SettingsSection>("daylite");

  if (!isOpen) {
    return null;
  }

  return (
    <dialog
      className="modal modal-open"
      open
      aria-labelledby="settings-dialog-title"
    >
      <section
        className="modal-box max-w-2xl p-0 flex overflow-hidden"
        style={{ minHeight: "420px" }}
      >
        <aside className="w-44 shrink-0 border-r border-base-300 p-2 flex flex-col gap-1 bg-base-200/40">
          <h2
            id="settings-dialog-title"
            className="px-3 py-2 text-xs font-semibold text-base-content/50 uppercase tracking-wide"
          >
            Einstellungen
          </h2>
          <button
            type="button"
            className={`btn btn-ghost btn-sm justify-start ${activeSection === "daylite" ? "btn-active" : ""}`}
            onClick={() => setActiveSection("daylite")}
          >
            Daylite
          </button>
          <button
            type="button"
            className={`btn btn-ghost btn-sm justify-start ${activeSection === "zep" ? "btn-active" : ""}`}
            onClick={() => setActiveSection("zep")}
          >
            ZEP
          </button>
        </aside>

        <main className="flex-1 p-6 overflow-y-auto">
          {activeSection === "daylite" ? (
            <DayliteSettingsPanel onClose={onClose} />
          ) : (
            <ZepSettingsPanel onClose={onClose} />
          )}
        </main>
      </section>

      <button
        type="button"
        className="modal-backdrop"
        onClick={onClose}
        aria-label="Einstellungen schließen"
      >
        Schließen
      </button>
    </dialog>
  );
}

function DayliteSettingsPanel({ onClose }: PanelProps) {
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

      {status ? (
        <section
          className={`alert mt-4 ${status.type === "success" ? "alert-success" : "alert-error"}`}
        >
          <span>{status.message}</span>
        </section>
      ) : null}

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

function ZepSettingsPanel({ onClose }: PanelProps) {
  const [rootUrlInput, setRootUrlInput] = useState("");
  const [usernameInput, setUsernameInput] = useState("");
  const [passwordInput, setPasswordInput] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [status, setStatus] = useState<PanelStatus | null>(null);

  useEffect(() => {
    setStatus(null);
    setPasswordInput("");
    let isActive = true;
    void loadZepCredentials().then((info: ZepCredentialsInfo | null) => {
      if (!isActive) {
        return;
      }
      if (info) {
        setRootUrlInput(info.rootUrl);
        setUsernameInput(info.username);
      }
    });
    return () => {
      isActive = false;
    };
  }, []);

  const onSubmit = async (event: ChangeEvent<HTMLFormElement>) => {
    event.preventDefault();
    setIsSaving(true);
    setStatus(null);

    const rootUrl = rootUrlInput.trim().replace(/\/+$/, "");
    const username = usernameInput.trim();

    if (!rootUrl) {
      setStatus({
        type: "error",
        message: "Bitte eine ZEP CalDAV-URL eingeben.",
      });
      setIsSaving(false);
      return;
    }
    if (!username) {
      setStatus({
        type: "error",
        message: "Bitte einen Benutzernamen eingeben.",
      });
      setIsSaving(false);
      return;
    }
    if (!passwordInput) {
      setStatus({ type: "error", message: "Bitte ein Passwort eingeben." });
      setIsSaving(false);
      return;
    }

    try {
      const testResult = await testZepCredentials(
        rootUrl,
        username,
        passwordInput,
      );
      await saveZepCredentials(rootUrl, username, passwordInput);
      setStatus({
        type: "success",
        message: `ZEP-Verbindung erfolgreich gespeichert. ${testResult.calendarCount} Kalender gefunden.`,
      });
      setPasswordInput("");
    } catch (error) {
      setStatus({
        type: "error",
        message:
          error instanceof Error
            ? error.message
            : "Die ZEP-Verbindung konnte nicht gespeichert werden.",
      });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <>
      <h3 className="text-lg font-semibold">ZEP-Verbindung</h3>

      {status ? (
        <section
          className={`alert mt-4 ${status.type === "success" ? "alert-success" : "alert-error"}`}
        >
          <span>{status.message}</span>
        </section>
      ) : null}

      <form className="mt-4 flex flex-col gap-4" onSubmit={onSubmit}>
        <label className="form-control w-full">
          <span className="label-text mb-2">ZEP CalDAV-URL</span>
          <input
            type="url"
            className="input input-bordered w-full"
            value={rootUrlInput}
            onChange={(event) => setRootUrlInput(event.target.value)}
            disabled={isSaving}
            placeholder="https://app.zep.de/caldav/admin"
          />
        </label>

        <label className="form-control w-full">
          <span className="label-text mb-2">Benutzername</span>
          <input
            type="text"
            className="input input-bordered w-full"
            value={usernameInput}
            onChange={(event) => setUsernameInput(event.target.value)}
            disabled={isSaving}
            placeholder="ZEP-Benutzername"
            autoComplete="username"
          />
        </label>

        <label className="form-control w-full">
          <span className="label-text mb-2">Passwort</span>
          <input
            type="password"
            className="input input-bordered w-full"
            value={passwordInput}
            onChange={(event) => setPasswordInput(event.target.value)}
            disabled={isSaving}
            placeholder="ZEP-Passwort"
            autoComplete="current-password"
          />
        </label>

        <section className="flex items-center justify-end gap-2">
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
            {isSaving ? "Verbinde..." : "Verbindung testen & speichern"}
          </button>
        </section>
      </form>
    </>
  );
}

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

interface PanelProps {
  onClose: () => void;
}

interface PanelStatus {
  type: "success" | "error";
  message: string;
}
