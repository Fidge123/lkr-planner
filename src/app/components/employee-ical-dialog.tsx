import { useEffect, useState } from "react";
import type {
  EmployeeSetting,
  IcalSource,
  PlanningContactRecord,
  ZepCalendar,
} from "../../generated/tauri";
import { saveAndTestCalendar } from "../../services/zep";

export function EmployeeIcalDialog({
  employee,
  employeeSetting,
  onClose,
  onSettingsSaved,
  zepCalendars,
  isLoadingCalendars,
  calendarsError,
  onReloadCalendars,
}: Props) {
  const isOpen = employee !== null;

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

  // Reset section state when dialog opens for a new employee
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
            calendars={zepCalendars}
            selectedUrl={primaryUrl}
            storedUrl={employeeSetting?.zepPrimaryCalendar ?? ""}
            onUrlChange={(url) => {
              setPrimaryUrl(url);
              setPrimaryStatus(null);
            }}
            status={primaryStatus ?? storedPrimaryStatus}
            isSubmitting={isPrimarySubmitting}
            onSubmit={() => void handleSubmit("primary")}
            isDisabled={isLoadingCalendars || zepCalendars === null}
          />

          <CalendarSection
            title="Abwesenheit"
            source="absence"
            calendars={zepCalendars}
            selectedUrl={absenceUrl}
            storedUrl={employeeSetting?.zepAbsenceCalendar ?? ""}
            onUrlChange={(url) => {
              setAbsenceUrl(url);
              setAbsenceStatus(null);
            }}
            status={absenceStatus ?? storedAbsenceStatus}
            isSubmitting={isAbsenceSubmitting}
            onSubmit={() => void handleSubmit("absence")}
            isDisabled={isLoadingCalendars || zepCalendars === null}
            isOptional
          />
        </section>

        <section className="mt-6 flex items-center justify-between gap-2">
          {calendarsError !== null ||
          (!isLoadingCalendars && zepCalendars !== null) ? (
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

export function CalendarSection({
  title,
  calendars,
  selectedUrl,
  storedUrl,
  onUrlChange,
  status,
  isSubmitting,
  onSubmit,
  isDisabled,
  isOptional,
}: CalendarSectionProps) {
  const isClearing = !selectedUrl && !!storedUrl;
  return (
    <section>
      <h3 className="font-semibold text-sm mb-2">
        {title}
        {isOptional ? (
          <span className="font-normal text-base-content/60 ml-1">
            (optional)
          </span>
        ) : null}
      </h3>

      <div className="flex items-end gap-2">
        <label className="form-control flex-1">
          <select
            className="select select-bordered select-sm w-full"
            value={selectedUrl}
            onChange={(e) => onUrlChange(e.target.value)}
            disabled={isDisabled || isSubmitting}
          >
            <option value="">— Kein Kalender —</option>
            {(calendars ?? []).map((cal) => (
              <option key={cal.url} value={cal.url}>
                {cal.displayName}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          className={`btn btn-sm whitespace-nowrap ${isClearing ? "btn-error btn-outline" : "btn-primary"}`}
          onClick={onSubmit}
          disabled={isDisabled || isSubmitting || (!selectedUrl && !storedUrl)}
        >
          {isSubmitting
            ? isClearing
              ? "Entferne..."
              : "Teste..."
            : isClearing
              ? "Entfernen"
              : "Speichern & Testen"}
        </button>
      </div>

      {isOptional && !selectedUrl && !status ? (
        <p className="text-xs text-base-content/60 mt-1">
          Ohne Abwesenheitskalender werden Abwesenheiten nicht synchronisiert.
        </p>
      ) : null}

      {status ? (
        <section
          className={`alert alert-sm mt-2 py-2 ${
            status.type === "success" ? "alert-success" : "alert-error"
          }`}
        >
          <span className="text-sm">{status.message}</span>
        </section>
      ) : (
        !isOptional &&
        !isDisabled &&
        !isSubmitting && (
          <p className="text-xs text-base-content/60 mt-1">Nicht getestet.</p>
        )
      )}
    </section>
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
  zepCalendars: ZepCalendar[] | null;
  isLoadingCalendars: boolean;
  calendarsError: string | null;
  onReloadCalendars: () => void;
}

export interface CalendarSectionProps {
  title: string;
  source: IcalSource;
  calendars: ZepCalendar[] | null;
  selectedUrl: string;
  storedUrl: string;
  onUrlChange: (url: string) => void;
  status: SectionStatus | null;
  isSubmitting: boolean;
  onSubmit: () => void;
  isDisabled: boolean;
  isOptional?: boolean;
}

export interface SectionStatus {
  type: "success" | "error";
  message: string;
}
