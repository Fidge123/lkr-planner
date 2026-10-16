import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app";
import { AppErrorBoundary } from "./app/components/error-boundary";
import { installGlobalErrorReporting } from "./app/global-error-reporting";

installGlobalErrorReporting(window);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppErrorBoundary>
      <App />
    </AppErrorBoundary>
  </React.StrictMode>,
);
