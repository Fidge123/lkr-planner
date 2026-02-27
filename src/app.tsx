import { useState } from "react";
import "./app.css";
import { ChevronLeft, ChevronRight, Settings } from "lucide-react";
import { DayliteTokenModal } from "./app/components/daylite-token-modal";
import { PlanningGrid } from "./app/page";

function App({ initialConfigOpen = false }: AppProps) {
  const [weekOffset, setWeekOffset] = useState(0);
  const [isDayliteTokenModalOpen, setIsDayliteTokenModalOpen] =
    useState(initialConfigOpen);

  const goToPreviousWeek = () => setWeekOffset((prev) => prev - 1);
  const goToNextWeek = () => setWeekOffset((prev) => prev + 1);
  const goToCurrentWeek = () => setWeekOffset(0);
  const openDayliteTokenModal = () => setIsDayliteTokenModalOpen(true);
  const closeDayliteTokenModal = () => setIsDayliteTokenModalOpen(false);

  return (
    <article className="min-h-screen flex flex-col">
      <header className="navbar p-4 shadow-sm border-b border-base-300">
        <div className="navbar-start gap-2">
          <h1 className="text-2xl font-bold">Wochenplanung</h1>
          <button
            type="button"
            className="btn btn-ghost px-2"
            onClick={openDayliteTokenModal}
          >
            <Settings className="size-6 text-base-content/50" />
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

      <DayliteTokenModal
        isOpen={isDayliteTokenModalOpen}
        onClose={closeDayliteTokenModal}
      />
    </article>
  );
}

export default App;

interface AppProps {
  initialConfigOpen?: boolean;
}
