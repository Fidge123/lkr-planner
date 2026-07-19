import { type ChangeEvent, useEffect, useState } from "react";
import type { ZepCredentialsInfo } from "../../../generated/tauri";
import {
  loadZepCredentials,
  saveZepCredentials,
  testZepCredentials,
} from "../../../services/zep";
import { type PanelStatus, StatusAlert } from "./panel-status";

export function ZepSettingsPanel({ onClose }: Props) {
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

      <StatusAlert status={status} />

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

interface Props {
  onClose: () => void;
}
