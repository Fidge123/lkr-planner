import { useEffect, useState } from "react";
import type {
  EmployeeSetting,
  IcalSource,
  PlanningContactRecord,
} from "../../generated/tauri";
import { saveAndTestCalendar } from "../../services/zep";
import type { ZepCalendarsState } from "../hooks/use-zep-calendars";
import { CalendarSection, type SectionStatus } from "./calendar-section";

export function EmployeeIcalDialog({
  employee,
  employeeSetting,
  onClose,
  onSettingsSaved,
  calendarState,
}: Props) {
  const isOpen = employee !== null;
  const {
    calendars,
    isLoading: isLoadingCalendars,
    errorMessage: calendarsError,
    reload: onReloadCalendars,
  } = calendarState;

  const [primaryUrl, setPrimaryUrl] = useState("");
  const [absenceUrl, setAbsenceUrl] = useState("");
  const [primaryStatus, setPrimaryStatus] = useState<SectionStatus | null>(
    null,
  );
  const [absenceStatus, setAbsenceStatus] = useState<SectionStatus | null>(
    null,
  );
  const [isPrimarySubmitting, setIsPrimarySubmitting] = useState(false);
  const [isAbsenceSubmitting, setIsAbsenceSubmitting] = useState(false);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    setPrimaryUrl(employeeSetting?.zepPrimaryCalendar ?? "");
    setAbsenceUrl(employeeSetting?.zepAbsenceCalendar ?? "");
    setPrimaryStatus(null);
    setAbsenceStatus(null);
  }, [isOpen, employeeSetting]);

  if (!isOpen || employee === null) {
    return null;
  }

  const employeeName =
    employee.nickname ?? employee.full_name ?? "Unbenannter Kontakt";

  const handleSubmit = async (source: IcalSource) => {
    const url = source === "primary" ? primaryUrl : absenceUrl;
    const setStatus =
      source === "primary" ? setPrimaryStatus : setAbsenceStatus;
    const setSubmitting =
      source === "primary" ? setIsPrimarySubmitting : setIsAbsenceSubmitting;

    setSubmitting(true);
    setStatus(null);

    try {
      const result = await saveAndTestCalendar(
        employee.self,
        source,
        url || null,
      );
      if (result.success) {
        setStatus({
          type: "success",
          message: url
            ? `Erfolgreich getestet am ${formatTimestamp(result.timestamp)}.`
            : "Kalender entfernt.",
        });
      } else {
        setStatus({
          type: "error",
          message:
            result.errorMessage ??
            "Verbindungstest fehlgeschlagen. Bitte prüfen.",
        });
      }
      onSettingsSaved();
    } catch (error) {
      setStatus({
        type: "error",
        message:
          error instanceof Error
            ? error.message
            : "Speichern und Testen fehlgeschlagen.",
      });
    } finally {
      setSubmitting(false);
    }
  };

  const storedPrimaryStatus = buildStoredStatus(
    employeeSetting?.primaryIcalLastTestedAt,
    employeeSetting?.primaryIcalLastTestPassed,
  );
  const storedAbsenceStatus = buildStoredStatus(
    employeeSetting?.absenceIcalLastTestedAt,
    employeeSetting?.absenceIcalLastTestPassed,
  );

  return (
    <dialog
      className="modal modal-open"
      open
      aria-labelledby="employee-ical-dialog-title"
    >
      <section className="modal-box max-w-xl">
        <h2 id="employee-ical-dialog-title" className="text-lg font-semibold">
          iCal-Konfiguration: {employeeName}
        </h2>

        <CalendarDiscoveryState
          isLoading={isLoadingCalendars}
          error={calendarsError}
          onReload={onReloadCalendars}
        />

        <section className="mt-5 flex flex-col gap-6">
          <CalendarSection
            title="Einsatz"
            source="primary"
            calendars={calendars}
            selectedUrl={primaryUrl}
            storedUrl={employeeSetting?.zepPrimaryCalendar ?? ""}
            onUrlChange={(url) => {
              setPrimaryUrl(url);
              setPrimaryStatus(null);
            }}
            status={primaryStatus ?? storedPrimaryStatus}
            isSubmitting={isPrimarySubmitting}
            onSubmit={() => void handleSubmit("primary")}
            isDisabled={isLoadingCalendars || calendars === null}
          />

          <CalendarSection
            title="Abwesenheit"
            source="absence"
            calendars={calendars}
            selectedUrl={absenceUrl}
            storedUrl={employeeSetting?.zepAbsenceCalendar ?? ""}
            onUrlChange={(url) => {
              setAbsenceUrl(url);
              setAbsenceStatus(null);
            }}
            status={absenceStatus ?? storedAbsenceStatus}
            isSubmitting={isAbsenceSubmitting}
            onSubmit={() => void handleSubmit("absence")}
            isDisabled={isLoadingCalendars || calendars === null}
            isOptional
          />
        </section>

        <section className="mt-6 flex items-center justify-between gap-2">
          {calendarsError !== null ||
          (!isLoadingCalendars && calendars !== null) ? (
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              onClick={onReloadCalendars}
            >
              Kalender neu laden
            </button>
          ) : (
            <span />
          )}
          <button
            type="button"
            className="btn btn-sm"
            onClick={onClose}
            disabled={isPrimarySubmitting || isAbsenceSubmitting}
          >
            Schließen
          </button>
        </section>
      </section>
      <button
        type="button"
        className="modal-backdrop"
        onClick={onClose}
        aria-label="Dialog schließen"
      >
        Schließen
      </button>
    </dialog>
  );
}

function CalendarDiscoveryState({
  isLoading,
  error,
  onReload,
}: {
  isLoading: boolean;
  error: string | null;
  onReload: () => void;
}) {
  if (isLoading) {
    return (
      <p className="text-sm text-base-content/60 mt-2">
        Kalender werden geladen...
      </p>
    );
  }
  if (error) {
    return (
      <section className="alert alert-error mt-3 py-2">
        <span className="text-sm">{error}</span>
        <button
          type="button"
          className="btn btn-ghost btn-xs ml-auto"
          onClick={onReload}
        >
          Neu laden
        </button>
      </section>
    );
  }
  return null;
}

function buildStoredStatus(
  lastTestedAt: string | null | undefined,
  lastTestPassed: boolean | null | undefined,
): SectionStatus | null {
  if (!lastTestedAt) {
    return null;
  }
  if (lastTestPassed === true) {
    return {
      type: "success",
      message: `Erfolgreich getestet am ${formatTimestamp(lastTestedAt)}.`,
    };
  }
  if (lastTestPassed === false) {
    return {
      type: "error",
      message: `Zuletzt fehlgeschlagen am ${formatTimestamp(lastTestedAt)}. Bitte erneut testen.`,
    };
  }
  return null;
}

function formatTimestamp(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return iso;
  }
  return date.toLocaleString("de-DE", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

interface Props {
  employee: PlanningContactRecord | null;
  employeeSetting: EmployeeSetting | null;
  onClose: () => void;
  onSettingsSaved: () => void;
  calendarState: ZepCalendarsState;
}
