import { commands, type Holiday } from "../generated/tauri";
import { callCommand } from "./command-result";

export function loadHolidaysForWeek(weekStart: string): Promise<Holiday[]> {
  return callCommand(
    commands.getHolidaysForWeek(weekStart),
    "Die Feiertage konnten nicht geladen werden.",
  );
}
