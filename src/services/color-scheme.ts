export type ColorScheme = "light" | "dark" | "system";

const storageKey = "colorScheme";
const colorSchemes: readonly ColorScheme[] = ["light", "dark", "system"];
const themeByScheme = { light: "corporate", dark: "business" } as const;

type SchemeStorage = Pick<Storage, "getItem" | "setItem">;
type ThemeRoot = Pick<Element, "setAttribute" | "removeAttribute">;

function defaultStorage(): SchemeStorage | undefined {
  try {
    return globalThis.localStorage;
  } catch {
    return undefined;
  }
}

function isColorScheme(value: string | null): value is ColorScheme {
  return colorSchemes.includes(value as ColorScheme);
}

export function loadColorScheme(
  storage: SchemeStorage | undefined = defaultStorage(),
): ColorScheme {
  try {
    const stored = storage?.getItem(storageKey) ?? null;
    return isColorScheme(stored) ? stored : "system";
  } catch {
    return "system";
  }
}

export function saveColorScheme(
  scheme: ColorScheme,
  storage: SchemeStorage | undefined = defaultStorage(),
): void {
  storage?.setItem(storageKey, scheme);
}

export function applyColorScheme(
  scheme: ColorScheme,
  root: ThemeRoot = document.documentElement,
): void {
  if (scheme === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", themeByScheme[scheme]);
  }
}
