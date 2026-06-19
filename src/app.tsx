import { useCallback, useEffect, useRef, useState } from "react";
import "./app.css";
import { ChevronLeft, ChevronRight, Settings } from "lucide-react";
import { EmployeeIcalDialog } from "./app/components/employee-ical-dialog";
import { SettingsDialog } from "./app/components/settings-dialog";
import { usePlanningAssignments } from "./app/hooks/use-planning-assignments";
import { PlanningGrid } from "./app/page";
import { getWeekDays } from "./app/util";
import type {
  EmployeeSetting,
  PlanningContactRecord,
  ZepCalendar,
} from "./generated/tauri";
import { commands } from "./generated/tauri";
import { loadDayliteContacts } from "./services/daylite-contacts";
import { discoverZepCalendars } from "./services/zep";

function App() {
  const [weekOffset, setWeekOffset] = useState(0);
  const weekStart = getWeekDays(weekOffset)[0].toISOString().slice(0, 10);
  const planningAssignmentsState = usePlanningAssignments(weekStart);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [icalDialogEmployee, setIcalDialogEmployee] =
    useState<PlanningContactRecord | null>(null);

  const [employeeSettings, setEmployeeSettings] = useState<EmployeeSetting[]>(
    [],
  );
  const [employeeSettingsError, setEmployeeSettingsError] = useState<
    string | null
  >(null);
  // Display preference: hide employees without a calendar or with category "Test".
  // Defaults to true (matches the backend DisplaySettings default).
  const [hideNonPlannableEmployees, setHideNonPlannableEmployees] =
    useState(true);
  // Session cache for discovered ZEP calendars (task 2.4). Not persisted across restarts;
  // null means "not yet fetched", [] means "fetched but empty".
  const [zepCalendars, setZepCalendars] = useState<ZepCalendar[] | null>(null);
  const [isLoadingCalendars, setIsLoadingCalendars] = useState(false);
  const [calendarsError, setCalendarsError] = useState<string | null>(null);

  const loadEmployeeSettings = useCallback(async () => {
    const result = await commands.loadLocalStore();
    if (result.status === "ok") {
      setEmployeeSettings(result.data.employeeSettings);
      setHideNonPlannableEmployees(
        result.data.displaySettings?.hideNonPlannableEmployees ?? true,
      );
      setEmployeeSettingsError(null);
    } else {
      setEmployeeSettingsError(result.error.userMessage);
    }
  }, []);

  // reloadAssignments depends on the visible week, but startup initialization must
  // run only once. A ref lets the effect call the latest version without re-running.
  const reloadAssignmentsRef = useRef(
    planningAssignmentsState.reloadAssignments,
  );
  reloadAssignmentsRef.current = planningAssignmentsState.reloadAssignments;

  // Daylite is the source of truth for an employee's calendar configuration.
  // On startup we sync contacts from Daylite first — this lets the backend
  // reconcile each employee's calendar URLs from Daylite into the local store —
  // and only then read the (now reconciled) settings. This is what makes a
  // calendar configured on one device show up on every other device. Assignments
  // are reloaded afterwards so events appear without a manual refresh.
  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        await loadDayliteContacts();
      } catch {
        // Daylite unreachable: fall back to whatever the local store already holds.
      }
      if (cancelled) return;
      await loadEmployeeSettings();
      if (cancelled) return;
      reloadAssignmentsRef.current();
    })();
    return () => {
      cancelled = true;
    };
  }, [loadEmployeeSettings]);

  const loadZepCalendars = useCallback(async () => {
    setIsLoadingCalendars(true);
    setCalendarsError(null);
    try {
      const calendars = await discoverZepCalendars();
      setZepCalendars(calendars);
    } catch (error) {
      setCalendarsError(
        error instanceof Error
          ? error.message
          : "Die ZEP-Kalender konnten nicht geladen werden.",
      );
    } finally {
      setIsLoadingCalendars(false);
    }
  }, []);

  const handleOpenIcalDialog = useCallback(
    (employee: PlanningContactRecord) => {
      setIcalDialogEmployee(employee);
      if (zepCalendars === null && !isLoadingCalendars) {
        void loadZepCalendars();
      }
    },
    [zepCalendars, isLoadingCalendars, loadZepCalendars],
  );

  const handleIcalDialogClose = () => {
    setIcalDialogEmployee(null);
  };

  const handleSettingsSaved = useCallback(() => {
    void loadEmployeeSettings();
    planningAssignmentsState.reloadAssignments();
  }, [loadEmployeeSettings, planningAssignmentsState.reloadAssignments]);

  return (
    <article className="min-h-screen flex flex-col" data-testid="planning-view">
      <header className="navbar p-4 shadow-sm border-b border-base-300">
        <div className="navbar-start gap-2">
          <h1 className="text-2xl font-bold">Wochenplanung</h1>
          <button
            type="button"
            className="btn btn-ghost px-2"
            onClick={() => setIsSettingsOpen(true)}
            aria-label="Einstellungen öffnen"
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
        {employeeSettingsError ? (
          <section className="alert alert-error m-4">
            <span>
              Einstellungen konnten nicht geladen werden:{" "}
              {employeeSettingsError}
            </span>
          </section>
        ) : null}
        <PlanningGrid
          weekOffset={weekOffset}
          assignmentState={planningAssignmentsState}
          employeeSettings={employeeSettings}
          hideNonPlannableEmployees={hideNonPlannableEmployees}
          onOpenIcalDialog={handleOpenIcalDialog}
        />
      </main>

      <SettingsDialog
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        onDisplaySettingsChanged={loadEmployeeSettings}
      />

      <EmployeeIcalDialog
        employee={icalDialogEmployee}
        employeeSetting={
          icalDialogEmployee
            ? (employeeSettings.find(
                (s) => s.dayliteContactReference === icalDialogEmployee.self,
              ) ?? null)
            : null
        }
        onClose={handleIcalDialogClose}
        onSettingsSaved={handleSettingsSaved}
        zepCalendars={zepCalendars}
        isLoadingCalendars={isLoadingCalendars}
        calendarsError={calendarsError}
        onReloadCalendars={loadZepCalendars}
      />
    </article>
  );
}

export default App;
