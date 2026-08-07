export function DataLoadingIndicator({ isLoading }: Props) {
  if (!isLoading) return null;

  return (
    <p
      className="flex items-center gap-2 text-sm text-base-content/70"
      aria-live="polite"
    >
      <span className="loading loading-spinner loading-sm" aria-hidden="true" />
      Daten werden geladen...
    </p>
  );
}

interface Props {
  isLoading: boolean;
}
