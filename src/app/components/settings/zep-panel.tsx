import { useQueryClient } from "@tanstack/react-query";
import { type ChangeEvent, useState } from "react";
import { saveZepCredentials, testZepCredentials } from "../../../services/zep";
import { usePanelSubmit } from "../../hooks/use-panel-submit";
import {
  useZepCredentials,
  zepCredentialsQuery,
} from "../../hooks/use-zep-credentials";
import { StatusAlert } from "./panel-status";

export function ZepSettingsPanel({ onClose }: Props) {
  const queryClient = useQueryClient();
  const { credentials, errorMessage } = useZepCredentials();
  const [rootUrlInput, setRootUrlInput] = useState<string | null>(null);
  const [usernameInput, setUsernameInput] = useState<string | null>(null);
  const [passwordInput, setPasswordInput] = useState("");
  const { isSaving, status, setStatus, run } = usePanelSubmit();
  const rootUrlValue = rootUrlInput ?? credentials?.rootUrl ?? "";
  const usernameValue = usernameInput ?? credentials?.username ?? "";

  const onSubmit = async (event: ChangeEvent<HTMLFormElement>) => {
    event.preventDefault();

    const rootUrl = rootUrlValue.trim().replace(/\/+$/, "");
    const username = usernameValue.trim();

    if (!rootUrl) {
      setStatus({
        type: "error",
        message: "Bitte eine ZEP CalDAV-URL eingeben.",
      });
      return;
    }
    if (!username) {
      setStatus({
        type: "error",
        message: "Bitte einen Benutzernamen eingeben.",
      });
      return;
    }
    if (!passwordInput) {
      setStatus({ type: "error", message: "Bitte ein Passwort eingeben." });
      return;
    }

    await run(async () => {
      const testResult = await testZepCredentials(
        rootUrl,
        username,
        passwordInput,
      );
      await saveZepCredentials(rootUrl, username, passwordInput);
      setPasswordInput("");
      await queryClient.invalidateQueries({
        queryKey: zepCredentialsQuery.queryKey,
      });
      return {
        type: "success",
        message: `ZEP-Verbindung erfolgreich gespeichert. ${testResult.calendarCount} Kalender gefunden.`,
      };
    }, "Die ZEP-Verbindung konnte nicht gespeichert werden.");
  };

  return (
    <>
      <h3 className="text-lg font-semibold">ZEP-Verbindung</h3>

      <StatusAlert
        status={
          status ??
          (errorMessage ? { type: "error", message: errorMessage } : null)
        }
      />

      <form className="mt-4 flex flex-col gap-4" onSubmit={onSubmit}>
        <label className="form-control w-full">
          <span className="label-text mb-2">ZEP CalDAV-URL</span>
          <input
            type="url"
            className="input input-bordered w-full"
            value={rootUrlValue}
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
            value={usernameValue}
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
