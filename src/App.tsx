import { useState } from "react";
import "./App.css";
import { PlanningGrid } from "./components/PlanningGrid";

function App() {
  const [weekOffset, setWeekOffset] = useState(0);

  const goToPreviousWeek = () => setWeekOffset((prev) => prev - 1);
  const goToNextWeek = () => setWeekOffset((prev) => prev + 1);
  const goToCurrentWeek = () => setWeekOffset(0);

  return (
    <article className="min-h-screen bg-base-100 flex flex-col">
      <header className="navbar bg-base-200 border-b border-base-300 shadow-sm">
        <nav className="navbar-start gap-4">
          <h1 className="text-2xl font-bold text-primary">LKR Planner</h1>
          <mark className="badge badge-outline badge-primary bg-transparent">
            Wochenansicht
          </mark>
        </nav>
        <nav className="navbar-end join">
          <button
            type="button"
            className="btn btn-ghost btn-sm join-item"
            onClick={goToPreviousWeek}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 19l-7-7 7-7"
              />
            </svg>
            Zurück
          </button>
          <button
            type="button"
            className={`btn btn-sm join-item ${weekOffset === 0 ? "btn-primary" : "btn-outline btn-primary"}`}
            onClick={goToCurrentWeek}
          >
            Heute
          </button>
          <button
            type="button"
            className="btn btn-ghost btn-sm join-item"
            onClick={goToNextWeek}
          >
            Weiter
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 5l7 7-7 7"
              />
            </svg>
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
