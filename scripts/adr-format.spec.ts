import { describe, expect, it } from "bun:test";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

describe("ADR documentation format", () => {
  const adrRoot = join(process.cwd(), "docs", "adr");

  it("uses consistent filenames and unique ADR identifiers", async () => {
    const files = (await readdir(adrRoot))
      .filter((fileName) => fileName.endsWith(".md"))
      .sort();

    expect(files.length).toBeGreaterThan(0);

    const seenIds = new Set<string>();
    for (const fileName of files) {
      expect(fileName).toMatch(/^\d{4}-[a-z0-9-]+\.md$/);

      const idFromFileName = fileName.slice(0, 4);
      expect(seenIds.has(idFromFileName)).toBe(false);
      seenIds.add(idFromFileName);

      const content = await readFile(join(adrRoot, fileName), "utf8");
      const titleMatch = content.match(/^# ADR (\d{4}): [^\n]+/m);
      expect(titleMatch).not.toBeNull();
      expect(titleMatch?.[1]).toBe(idFromFileName);
    }
  });

  it("uses required ADR sections and metadata fields", async () => {
    const files = (await readdir(adrRoot))
      .filter((fileName) => fileName.endsWith(".md"))
      .sort();

    for (const fileName of files) {
      const content = await readFile(join(adrRoot, fileName), "utf8");

      expect(content).toMatch(/^- Status: [^\n]+$/m);
      expect(content).toMatch(/^- Date: \d{4}-\d{2}-\d{2}$/m);
      expect(content).toContain("## Context");
      expect(content).toContain("## Decision");
      expect(content).toContain("## Consequences");
    }
  });
});
