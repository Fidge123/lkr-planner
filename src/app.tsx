import { useState } from "react";
import "./app.css";
import { ChevronLeft, ChevronRight, Settings } from "lucide-react";
import { DayliteTokenModal } from "./app/components/daylite-token-modal";
import { PlanningGrid } from "./app/page";

function App() {
  const [weekOffset, setWeekOffset] = useState(0);
  const [isDayliteTokenModalOpen, setIsDayliteTokenModalOpen] = useState(false);

  return (
    <article className="min-h-screen flex flex-col">
      <header className="navbar p-4 shadow-sm border-b border-base-300">
        <div className="navbar-start gap-2">
          <h1 className="text-2xl font-bold">Wochenplanung</h1>
          <button
            type="button"
            className="btn btn-ghost px-2"
            onClick={() => setIsDayliteTokenModalOpen(true)}
          >
            <Settings className="size-6 text-base-content/50" />
          </button>
        </div>
        <nav className="navbar-end gap-2">
          <button
            type="button"
            className="btn btn-ghost pl-2"
            onClick={() => setWeekOffset((prev) => prev - 1)}
          >
            <ChevronLeft className="" />
            Zurück
          </button>
          <button
            type="button"
            className={`btn px-6 btn-primary ${weekOffset !== 0 && "btn-outline"}`}
            onClick={() => setWeekOffset(0)}
          >
            Heute
          </button>
          <button
            type="button"
            className="btn btn-ghost pr-2"
            onClick={() => setWeekOffset((prev) => prev + 1)}
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
        onClose={() => setIsDayliteTokenModalOpen(false)}
      />
    </article>
  );
}

export default App;
