export function StatusAlert({ status }: Props) {
  if (!status) {
    return null;
  }
  return (
    <section
      className={`alert mt-4 ${status.type === "success" ? "alert-success" : "alert-error"}`}
    >
      <span>{status.message}</span>
    </section>
  );
}

export interface PanelStatus {
  type: "success" | "error";
  message: string;
}

interface Props {
  status: PanelStatus | null;
}
