import { QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app";
import { AppErrorBoundary } from "./app/components/error-boundary";
import { installGlobalErrorReporting } from "./app/global-error-reporting";
import { installSettledOnlineManager, queryClient } from "./app/query-client";
import { applyColorScheme, loadColorScheme } from "./services/color-scheme";

installGlobalErrorReporting(window);
installSettledOnlineManager(window);
applyColorScheme(loadColorScheme());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    </AppErrorBoundary>
  </React.StrictMode>,
);
