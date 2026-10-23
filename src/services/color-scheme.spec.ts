import { describe, expect, it } from "bun:test";
import {
  applyColorScheme,
  loadColorScheme,
  saveColorScheme,
} from "./color-scheme";

function createStorage(initial: Record<string, string> = {}) {
  const values = new Map(Object.entries(initial));
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value);
    },
  };
}

function createRoot() {
  const attributes = new Map<string, string>();
  return {
    attributes,
    setAttribute: (name: string, value: string) => {
      attributes.set(name, value);
    },
    removeAttribute: (name: string) => {
      attributes.delete(name);
    },
  };
}

describe("color scheme service", () => {
  it("defaults to system when nothing is stored", () => {
    expect(loadColorScheme(createStorage())).toBe("system");
  });

  it("defaults to system when the stored value is unknown", () => {
    expect(loadColorScheme(createStorage({ colorScheme: "sepia" }))).toBe(
      "system",
    );
  });

  it("persists the selected scheme", () => {
    const storage = createStorage();
    saveColorScheme("dark", storage);
    expect(loadColorScheme(storage)).toBe("dark");
  });

  it("applies the light theme", () => {
    const root = createRoot();
    applyColorScheme("light", root);
    expect(root.attributes.get("data-theme")).toBe("corporate");
  });

  it("applies the dark theme", () => {
    const root = createRoot();
    applyColorScheme("dark", root);
    expect(root.attributes.get("data-theme")).toBe("business");
  });

  it("defers to the operating system for system", () => {
    const root = createRoot();
    applyColorScheme("dark", root);
    applyColorScheme("system", root);
    expect(root.attributes.has("data-theme")).toBe(false);
  });
});
