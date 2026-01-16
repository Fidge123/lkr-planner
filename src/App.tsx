import "./App.css";
import { PlanningGrid } from "./components/PlanningGrid";

function App() {
  return (
    <div className="min-h-screen bg-base-100 flex flex-col">
      {/* Header */}
      <header className="bg-base-200 border-b border-base-300 px-6 py-4 flex items-center justify-between shadow-sm">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-bold text-primary">LKR Planner</h1>
          <div className="badge badge-outline badge-primary">Week View</div>
        </div>
        <div className="flex items-center gap-2">
          <button type="button" className="btn btn-ghost btn-sm">
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
            Previous
          </button>
          <button type="button" className="btn btn-primary btn-sm">
            Today
          </button>
          <button type="button" className="btn btn-ghost btn-sm">
            Next
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
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 overflow-hidden">
        <PlanningGrid />
      </main>
    </div>
  );
}

export default App;
