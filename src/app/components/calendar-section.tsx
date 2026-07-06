import type { IcalSource, ZepCalendar } from "../../generated/tauri";

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
