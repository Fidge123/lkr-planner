import { useQueryClient } from "@tanstack/react-query";
import { ExternalLink } from "lucide-react";
import { type ChangeEvent, useState } from "react";
import {
  daylitePersonalTokenUrl,
  updateDayliteRefreshToken,
} from "../../../services/daylite-auth";
import { localStoreQuery, useLocalStore } from "../../hooks/use-local-store";
import { usePanelSubmit } from "../../hooks/use-panel-submit";
import { StatusAlert } from "./panel-status";

export function DayliteSettingsPanel({ onClose }: Props) {
  const queryClient = useQueryClient();
  const { dayliteBaseUrl } = useLocalStore();
  const [baseUrlInput, setBaseUrlInput] = useState<string | null>(null);
  const [refreshTokenInput, setRefreshTokenInput] = useState("");
  const { isSaving, status, run } = usePanelSubmit();
  const baseUrl = baseUrlInput ?? dayliteBaseUrl;

  const onSubmit = async (event: ChangeEvent<HTMLFormElement>) => {
    event.preventDefault();
    await run(async () => {
      await updateDayliteRefreshToken({
        baseUrl,
        refreshToken: refreshTokenInput,
      });
      setRefreshTokenInput("");
      await queryClient.invalidateQueries({
        queryKey: localStoreQuery.queryKey,
      });
      return {
        type: "success",
        message: "Daylite-Konfiguration wurde aktualisiert.",
      };
    }, "Das Daylite-Refresh-Token konnte nicht gespeichert werden.");
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
            value={baseUrl}
            onChange={(event) => setBaseUrlInput(event.target.value)}
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
            href={daylitePersonalTokenUrl}
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
