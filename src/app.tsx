import { useIsFetching, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";
import "./app.css";
import { ChevronLeft, ChevronRight, Settings } from "lucide-react";
import { DataLoadingIndicator } from "./app/components/data-loading-indicator";
import { EmployeeIcalDialog } from "./app/components/employee-ical-dialog";
import { SettingsDialog } from "./app/components/settings/settings-dialog";
import { useCalendarAutoRefresh } from "./app/hooks/use-calendar-auto-refresh";
import { localStoreQuery, useLocalStore } from "./app/hooks/use-local-store";
import { usePlanningAssignments } from "./app/hooks/use-planning-assignments";
import { contactsQuery } from "./app/hooks/use-planning-employees";
import { useReloadDataMenu } from "./app/hooks/use-reload-data-menu";
import { useRerenderOnDayChange } from "./app/hooks/use-rerender-on-day-change";
import { useZepCalendars } from "./app/hooks/use-zep-calendars";
import { PlanningGrid } from "./app/page";
import { runStartupSync } from "./app/startup-sync";
import { getWeekStart, weekOffsetFor } from "./app/util";
import type { PlanningContactRecord } from "./generated/tauri";

function App() {
  const [weekOffset, setWeekOffset] = useState(0);
  const localStore = useLocalStore();
  const { showWeekend, hideNonPlannableEmployees } = localStore.displaySettings;
  useRerenderOnDayChange();
  const weekStart = getWeekStart(weekOffset, showWeekend);
  // On a weekend the weekend setting decides which week is shown, so no week loads before it is read.
  const planningAssignmentsState = usePlanningAssignments(
    localStore.isLoaded ? weekStart : null,
  );
  useCalendarAutoRefresh(
    planningAssignmentsState.lastLoadedAt,
    planningAssignmentsState.refreshAssignments,
  );
  const queryClient = useQueryClient();
  const isLoadingEmployees =
    useIsFetching({ queryKey: contactsQuery.queryKey }) > 0;
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [icalDialogEmployee, setIcalDialogEmployee] =
    useState<PlanningContactRecord | null>(null);

  const zepCalendarsState = useZepCalendars(icalDialogEmployee !== null);

  const reloadAssignmentsRef = useRef(
    planningAssignmentsState.reloadAssignments,
  );
  reloadAssignmentsRef.current = planningAssignmentsState.reloadAssignments;

  useEffect(() => {
    let cancelled = false;
    void runStartupSync({
      readSettings: () =>
        queryClient.fetchQuery({ ...localStoreQuery, staleTime: 0 }).then(
          (store) => store.employeeSettings,
          () => null,
        ),
      syncContacts: async () => {
        await queryClient.fetchQuery(contactsQuery);
      },
      reloadWeek: () => reloadAssignmentsRef.current(),
      isCancelled: () => cancelled,
    });
    return () => {
      cancelled = true;
    };
  }, [queryClient]);

  const handleIcalDialogClose = () => {
    setIcalDialogEmployee(null);
  };

  const handleNavigateWeek = useCallback((direction: -1 | 1) => {
    setWeekOffset((prev) => prev + direction);
  }, []);

  const handleJumpToWeek = useCallback(
    (day: Date) => setWeekOffset(weekOffsetFor(day, showWeekend)),
    [showWeekend],
  );

  const reloadData = useCallback(() => {
    void queryClient.invalidateQueries();
    planningAssignmentsState.reloadAssignments();
  }, [queryClient, planningAssignmentsState.reloadAssignments]);

  useReloadDataMenu(reloadData);

  const handleSettingsSaved = useCallback(() => {
    void queryClient.invalidateQueries({ queryKey: localStoreQuery.queryKey });
    planningAssignmentsState.reloadAssignments();
  }, [queryClient, planningAssignmentsState.reloadAssignments]);

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
            isLoading={planningAssignmentsState.isLoading || isLoadingEmployees}
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
        {localStore.errorMessage ? (
          <section className="alert alert-error m-4">
            <span>
              Einstellungen konnten nicht geladen werden:{" "}
              {localStore.errorMessage}
            </span>
          </section>
        ) : null}
        <PlanningGrid
          weekOffset={weekOffset}
          showWeekend={showWeekend}
          assignmentState={planningAssignmentsState}
          employeeSettings={localStore.employeeSettings}
          hideNonPlannableEmployees={hideNonPlannableEmployees}
          onOpenIcalDialog={setIcalDialogEmployee}
          onNavigateWeek={handleNavigateWeek}
          onJumpToWeek={handleJumpToWeek}
        />
      </main>

      <SettingsDialog
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
      />

      <EmployeeIcalDialog
        employee={icalDialogEmployee}
        employeeSetting={
          icalDialogEmployee
            ? (localStore.employeeSettings.find(
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
