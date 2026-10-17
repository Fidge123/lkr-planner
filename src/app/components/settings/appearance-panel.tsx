import { type LucideIcon, Monitor, Moon, Sun } from "lucide-react";
import { useState } from "react";
import {
  applyColorScheme,
  type ColorScheme,
  loadColorScheme,
  saveColorScheme,
} from "../../../services/color-scheme";
import { StatusAlert } from "./panel-status";

const options: { value: ColorScheme; label: string; icon: LucideIcon }[] = [
  { value: "light", label: "Hell", icon: Sun },
  { value: "dark", label: "Dunkel", icon: Moon },
  { value: "system", label: "System", icon: Monitor },
];

export function AppearanceSettingsPanel({ onClose }: Props) {
  const [colorScheme, setColorScheme] = useState(() => loadColorScheme());
  const [hasError, setHasError] = useState(false);

  const selectColorScheme = (scheme: ColorScheme) => {
    setColorScheme(scheme);
    applyColorScheme(scheme);
    try {
      saveColorScheme(scheme);
      setHasError(false);
    } catch {
      setHasError(true);
    }
  };

  return (
    <>
      <h3 className="text-lg font-semibold">Erscheinungsbild</h3>

      <StatusAlert
        status={
          hasError
            ? {
                type: "error",
                message: "Das Farbschema konnte nicht gespeichert werden.",
              }
            : null
        }
      />

      <fieldset className="fieldset mt-4">
        <legend className="fieldset-legend">Farbschema</legend>
        {options.map(({ value, label, icon: Icon }) => (
          <label
            key={value}
            className="label cursor-pointer justify-start gap-3"
          >
            <input
              type="radio"
              name="color-scheme"
              className="radio radio-primary"
              value={value}
              checked={colorScheme === value}
              onChange={() => selectColorScheme(value)}
            />
            <Icon className="size-4" aria-hidden />
            <span className="label-text font-medium">{label}</span>
          </label>
        ))}
      </fieldset>

      <section className="mt-6 flex items-center justify-end">
        <button type="button" className="btn btn-sm" onClick={onClose}>
          Schließen
        </button>
      </section>
    </>
  );
}

interface Props {
  onClose: () => void;
}
