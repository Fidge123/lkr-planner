export function AppointmentSyncAlert({
  failedDates,
  isRetrying,
  onRetry,
}: Props) {
  if (failedDates.length === 0) return null;

  const days = [...failedDates].sort().map(formatGermanDate).join(", ");
  return (
    <section className="alert alert-error m-4">
      <span>
        Daylite-Termine für folgende Tage konnten nicht aktualisiert werden:{" "}
        {days}
      </span>
      <button
        type="button"
        className="btn btn-sm"
        disabled={isRetrying}
        onClick={onRetry}
      >
        {isRetrying ? "Wird wiederholt..." : "Erneut versuchen"}
      </button>
    </section>
  );
}

function formatGermanDate(isoDate: string): string {
  const [year, month, day] = isoDate.split("-");
  return `${day}.${month}.${year}`;
}

interface Props {
  failedDates: string[];
  isRetrying: boolean;
  onRetry: () => void;
}
