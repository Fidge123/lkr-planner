import { useState } from "react";
import "./app.css";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { PlanningGrid } from "./app/page";

function App() {
  const [weekOffset, setWeekOffset] = useState(0);

  const goToPreviousWeek = () => setWeekOffset((prev) => prev - 1);
  const goToNextWeek = () => setWeekOffset((prev) => prev + 1);
  const goToCurrentWeek = () => setWeekOffset(0);

  return (
    <article className="min-h-screen bg-base-100 flex flex-col">
      <header className="navbar p-4 shadow-sm border-b border-slate-300">
        <h1 className="navbar-start text-2xl font-bold">Wochenplanung</h1>
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
    </article>
  );
}

export default App;
