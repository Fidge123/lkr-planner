import { useCallback, useEffect, useState } from "react";
import "./app.css";
import { ChevronLeft, ChevronRight, Settings } from "lucide-react";
import { DayliteTokenModal } from "./app/components/daylite-token-modal";
import { EmployeeIcalDialog } from "./app/components/employee-ical-dialog";
import { ZepCredentialsModal } from "./app/components/zep-credentials-modal";
import { PlanningGrid } from "./app/page";
import type {
  EmployeeSetting,
  PlanningContactRecord,
  ZepCalendar,
} from "./generated/tauri";
import { commands } from "./generated/tauri";
import { discoverZepCalendars } from "./services/zep";

function App() {
  const [weekOffset, setWeekOffset] = useState(0);
  const [isDayliteTokenModalOpen, setIsDayliteTokenModalOpen] = useState(false);
  const [isZepCredentialsModalOpen, setIsZepCredentialsModalOpen] =
    useState(false);
  const [icalDialogEmployee, setIcalDialogEmployee] =
    useState<PlanningContactRecord | null>(null);

  const [employeeSettings, setEmployeeSettings] = useState<EmployeeSetting[]>(
    [],
  );
  const [zepCalendars, setZepCalendars] = useState<ZepCalendar[] | null>(null);
  const [isLoadingCalendars, setIsLoadingCalendars] = useState(false);
  const [calendarsError, setCalendarsError] = useState<string | null>(null);

  const loadEmployeeSettings = useCallback(async () => {
    const result = await commands.loadLocalStore();
    if (result.status === "ok") {
      setEmployeeSettings(result.data.employeeSettings);
    }
  }, []);

  useEffect(() => {
    void loadEmployeeSettings();
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
  }, [loadEmployeeSettings]);

  return (
    <article className="min-h-screen flex flex-col">
      <header className="navbar p-4 shadow-sm border-b border-base-300">
        <div className="navbar-start gap-2">
          <h1 className="text-2xl font-bold">Wochenplanung</h1>
          <div className="dropdown">
            <button
              type="button"
              className="btn btn-ghost px-2"
              aria-label="Einstellungen öffnen"
            >
              <Settings className="size-6 text-base-content/50" />
            </button>
            <ul className="dropdown-content menu bg-base-100 rounded-box z-10 w-52 p-2 shadow border border-base-300">
              <li>
                <button
                  type="button"
                  onClick={() => setIsDayliteTokenModalOpen(true)}
                >
                  Daylite-Konfiguration
                </button>
              </li>
              <li>
                <button
                  type="button"
                  onClick={() => setIsZepCredentialsModalOpen(true)}
                >
                  ZEP-Verbindung
                </button>
              </li>
            </ul>
          </div>
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
        <PlanningGrid
          weekOffset={weekOffset}
          employeeSettings={employeeSettings}
          onOpenIcalDialog={handleOpenIcalDialog}
        />
      </main>

      <DayliteTokenModal
        isOpen={isDayliteTokenModalOpen}
        onClose={() => setIsDayliteTokenModalOpen(false)}
      />

      <ZepCredentialsModal
        isOpen={isZepCredentialsModalOpen}
        onClose={() => setIsZepCredentialsModalOpen(false)}
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
