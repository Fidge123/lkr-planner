import { useCallback, useEffect, useRef, useState } from "react";
import "./app.css";
import { ChevronLeft, ChevronRight, Settings } from "lucide-react";
import { DataLoadingIndicator } from "./app/components/data-loading-indicator";
import { EmployeeIcalDialog } from "./app/components/employee-ical-dialog";
import { SettingsDialog } from "./app/components/settings/settings-dialog";
import { usePlanningAssignments } from "./app/hooks/use-planning-assignments";
import { usePlanningEmployees } from "./app/hooks/use-planning-employees";
import { useProjectCategoryColors } from "./app/hooks/use-project-category-colors";
import { useReloadDataMenu } from "./app/hooks/use-reload-data-menu";
import { useRerenderOnDayChange } from "./app/hooks/use-rerender-on-day-change";
import { useZepCalendars } from "./app/hooks/use-zep-calendars";
import { PlanningGrid } from "./app/page";
import { getWeekStart } from "./app/util";
import type { EmployeeSetting, PlanningContactRecord } from "./generated/tauri";
import { commands } from "./generated/tauri";
import { resetProjectCategoryColors } from "./services/daylite-categories";
import { loadDayliteContacts } from "./services/daylite-contacts";

function App() {
  const [weekOffset, setWeekOffset] = useState(0);
  const [showWeekend, setShowWeekend] = useState(false);
  useRerenderOnDayChange();
  const weekStart = getWeekStart(weekOffset, showWeekend);
  const planningAssignmentsState = usePlanningAssignments(weekStart);
  const planningEmployeesState = usePlanningEmployees();
  const categoryColors = useProjectCategoryColors();
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [icalDialogEmployee, setIcalDialogEmployee] =
    useState<PlanningContactRecord | null>(null);

  const [employeeSettings, setEmployeeSettings] = useState<EmployeeSetting[]>(
    [],
  );
  const [employeeSettingsError, setEmployeeSettingsError] = useState<
    string | null
  >(null);
  const [hideNonPlannableEmployees, setHideNonPlannableEmployees] =
    useState(true);
  const zepCalendarsState = useZepCalendars();

  const loadEmployeeSettings = useCallback(async () => {
    const result = await commands.loadLocalStore();
    if (result.status === "ok") {
      setEmployeeSettings(result.data.employeeSettings);
      setHideNonPlannableEmployees(
        result.data.displaySettings?.hideNonPlannableEmployees ?? true,
      );
      setShowWeekend(result.data.displaySettings?.showWeekend ?? false);
      setEmployeeSettingsError(null);
    } else {
      setEmployeeSettingsError(result.error.userMessage);
    }
  }, []);

  const reloadAssignmentsRef = useRef(
    planningAssignmentsState.reloadAssignments,
  );
  reloadAssignmentsRef.current = planningAssignmentsState.reloadAssignments;

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        await loadDayliteContacts();
      } catch {}
      if (cancelled) return;
      await loadEmployeeSettings();
      if (cancelled) return;
      reloadAssignmentsRef.current();
    })();
    return () => {
      cancelled = true;
    };
  }, [loadEmployeeSettings]);

  const handleOpenIcalDialog = useCallback(
    (employee: PlanningContactRecord) => {
      setIcalDialogEmployee(employee);
      zepCalendarsState.ensureLoaded();
    },
    [zepCalendarsState.ensureLoaded],
  );

  const handleIcalDialogClose = () => {
    setIcalDialogEmployee(null);
  };

  const handleNavigateWeek = useCallback((direction: -1 | 1) => {
    setWeekOffset((prev) => prev + direction);
  }, []);

  const reloadData = useCallback(() => {
    resetProjectCategoryColors();
    void loadEmployeeSettings();
    planningEmployeesState.reloadEmployees();
    planningAssignmentsState.reloadAssignments();
  }, [
    loadEmployeeSettings,
    planningEmployeesState.reloadEmployees,
    planningAssignmentsState.reloadAssignments,
  ]);

  useReloadDataMenu(reloadData);

  const handleSettingsSaved = useCallback(() => {
    void loadEmployeeSettings();
    planningAssignmentsState.reloadAssignments();
  }, [loadEmployeeSettings, planningAssignmentsState.reloadAssignments]);

  return (
    <article className="h-screen flex flex-col">
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
          <DataLoadingIndicator
            isLoading={
              planningAssignmentsState.isLoading ||
              planningEmployeesState.isLoading
            }
          />
        </div>
        <nav className="navbar-end gap-2">
          <button
            type="button"
            className="btn btn-ghost pl-2"
            onClick={() => handleNavigateWeek(-1)}
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
            onClick={() => handleNavigateWeek(1)}
          >
            Weiter
            <ChevronRight />
          </button>
        </nav>
      </header>

      <main className="flex-1 min-h-0 overflow-hidden">
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
          showWeekend={showWeekend}
          assignmentState={planningAssignmentsState}
          employeeState={planningEmployeesState}
          employeeSettings={employeeSettings}
          categoryColors={categoryColors}
          hideNonPlannableEmployees={hideNonPlannableEmployees}
          onOpenIcalDialog={handleOpenIcalDialog}
          onNavigateWeek={handleNavigateWeek}
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
        calendarState={zepCalendarsState}
      />
    </article>
  );
}

export default App;
