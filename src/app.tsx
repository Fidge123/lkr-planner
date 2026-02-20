import { type ChangeEvent, type FormEvent, useState } from "react";
import "./app.css";
import {
  ChevronLeft,
  ChevronRight,
  ExternalLink,
  Settings,
} from "lucide-react";
import { PlanningGrid } from "./app/page";
import {
  DAYLITE_PERSONAL_TOKEN_URL,
  updateDayliteRefreshToken,
} from "./services/daylite-auth";

function App() {
  const [weekOffset, setWeekOffset] = useState(0);
  const [isDayliteTokenModalOpen, setIsDayliteTokenModalOpen] = useState(false);
  const [refreshTokenInput, setRefreshTokenInput] = useState("");
  const [isSavingRefreshToken, setIsSavingRefreshToken] = useState(false);
  const [refreshTokenStatus, setRefreshTokenStatus] =
    useState<RefreshTokenStatus | null>(null);

  const goToPreviousWeek = () => setWeekOffset((prev) => prev - 1);
  const goToNextWeek = () => setWeekOffset((prev) => prev + 1);
  const goToCurrentWeek = () => setWeekOffset(0);
  const openDayliteTokenModal = () => {
    setRefreshTokenStatus(null);
    setRefreshTokenInput("");
    setIsDayliteTokenModalOpen(true);
  };
  const closeDayliteTokenModal = () => {
    if (isSavingRefreshToken) {
      return;
    }

    setIsDayliteTokenModalOpen(false);
  };
  const onRefreshTokenInputChange = (event: ChangeEvent<HTMLInputElement>) => {
    setRefreshTokenInput(event.target.value);
  };
  const onRefreshTokenSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setIsSavingRefreshToken(true);
    setRefreshTokenStatus(null);

    try {
      await updateDayliteRefreshToken(refreshTokenInput);
      setRefreshTokenStatus({
        type: "success",
        message: "Daylite-Refresh-Token wurde aktualisiert.",
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

  return (
    <article className="min-h-screen bg-base-100 flex flex-col">
      <header className="navbar p-4 shadow-sm border-b border-slate-300">
        <div className="navbar-start gap-2">
          <h1 className="text-2xl font-bold">Wochenplanung</h1>
          <button
            type="button"
            className="btn btn-ghost px-2"
            onClick={openDayliteTokenModal}
          >
            <Settings className="size-6" />
          </button>
        </div>
        <nav className="navbar-end gap-2">
          <button
            type="button"
            className="btn btn-ghost pl-2"
            onClick={goToPreviousWeek}
          >
            <ChevronLeft className="" />
            Zurück
          </button>
          <button
            type="button"
            className={`btn px-6 btn-primary ${weekOffset !== 0 && "btn-outline"}`}
            onClick={goToCurrentWeek}
          >
            Heute
          </button>
          <button
            type="button"
            className="btn btn-ghost pr-2"
            onClick={goToNextWeek}
          >
            Weiter
            <ChevronRight />
          </button>
        </nav>
      </header>

      <main className="flex-1 overflow-hidden">
        <PlanningGrid weekOffset={weekOffset} />
      </main>

      {isDayliteTokenModalOpen ? (
        <div className="modal modal-open">
          <section className="modal-box max-w-xl">
            <h2 className="text-lg font-semibold">
              Daylite-Refresh-Token aktualisieren
            </h2>
            <p className="mt-2 text-sm text-base-content/80">
              Trage ein neues Refresh-Token ein, damit Daylite erneut verbunden
              wird.
            </p>

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
                <span className="label-text mb-2">Refresh-Token</span>
                <input
                  type="password"
                  className="input input-bordered w-full"
                  value={refreshTokenInput}
                  onChange={onRefreshTokenInputChange}
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
                    onClick={closeDayliteTokenModal}
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
            onClick={closeDayliteTokenModal}
            aria-label="Dialog schließen"
          >
            schließen
          </button>
        </div>
      ) : null}
    </article>
  );
}

export default App;

interface RefreshTokenStatus {
  type: "success" | "error";
  message: string;
}
