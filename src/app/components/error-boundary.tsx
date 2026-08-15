import { Component, type ErrorInfo, type ReactNode } from "react";
import { reportFrontendError } from "../../services/telemetry";

export class AppErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    void reportFrontendError({
      source: "render",
      name: error.name,
      message: error.message,
      context: topComponent(errorInfo.componentStack),
    });
  }

  render() {
    if (!this.state.hasError) {
      return this.props.children;
    }

    return (
      <section className="hero min-h-screen bg-base-200">
        <section className="hero-content text-center">
          <section className="max-w-md">
            <h1 className="text-2xl font-bold">
              Es ist ein unerwarteter Fehler aufgetreten
            </h1>
            <p className="mt-4 text-base-content/70">
              Bitte starten Sie die Anwendung neu. Wenn der Fehler erneut
              auftritt, wenden Sie sich an den Support.
            </p>
          </section>
        </section>
      </section>
    );
  }
}

function topComponent(
  componentStack: string | null | undefined,
): string | null {
  return (
    componentStack
      ?.split("\n")
      .map((line) => line.trim().replace(/^at\s+/, ""))
      .find((line) => line.length > 0) ?? null
  );
}

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
}
