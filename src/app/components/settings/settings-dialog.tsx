import { useState } from "react";
import { DayliteSettingsPanel } from "./daylite-panel";
import { DisplaySettingsPanel } from "./display-panel";
import { ZepSettingsPanel } from "./zep-panel";

const sections = [
  { id: "daylite", label: "Daylite" },
  { id: "zep", label: "ZEP" },
  { id: "display", label: "Anzeige" },
] as const;

type SettingsSection = (typeof sections)[number]["id"];

export function SettingsDialog({
  isOpen,
  onClose,
  onDisplaySettingsChanged,
}: Props) {
  const [activeSection, setActiveSection] =
    useState<SettingsSection>("daylite");

  if (!isOpen) {
    return null;
  }

  return (
    <dialog
      className="modal modal-open"
      open
      aria-labelledby="settings-dialog-title"
    >
      <section
        className="modal-box max-w-2xl p-0 flex overflow-hidden"
        style={{ minHeight: "420px" }}
      >
        <aside className="w-44 shrink-0 border-r border-base-300 p-2 flex flex-col gap-1 bg-base-200/40">
          <h2
            id="settings-dialog-title"
            className="px-3 py-2 text-xs font-semibold text-base-content/50 uppercase tracking-wide"
          >
            Einstellungen
          </h2>
          {sections.map((section) => (
            <button
              key={section.id}
              type="button"
              className={`btn btn-ghost btn-sm justify-start ${activeSection === section.id ? "btn-active" : ""}`}
              onClick={() => setActiveSection(section.id)}
            >
              {section.label}
            </button>
          ))}
        </aside>

        <main className="flex-1 p-6 overflow-y-auto">
          {activeSection === "daylite" ? (
            <DayliteSettingsPanel onClose={onClose} />
          ) : activeSection === "zep" ? (
            <ZepSettingsPanel onClose={onClose} />
          ) : (
            <DisplaySettingsPanel
              onClose={onClose}
              onChanged={onDisplaySettingsChanged}
            />
          )}
        </main>
      </section>

      <button
        type="button"
        className="modal-backdrop"
        onClick={onClose}
        aria-label="Einstellungen schließen"
      >
        Schließen
      </button>
    </dialog>
  );
}

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onDisplaySettingsChanged?: () => void;
}
