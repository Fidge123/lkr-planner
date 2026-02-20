import { describe, expect, it } from "bun:test";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

interface EpicEntry {
  dirName: string;
  id: string;
}

async function loadEpicEntries(backlogRoot: string): Promise<EpicEntry[]> {
  const entries = await readdir(backlogRoot, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isDirectory() && entry.name.startsWith("epic-"))
    .map((entry) => {
      const match = entry.name.match(/^epic-(\d{2})-[a-z0-9-]+$/);
      expect(match).not.toBeNull();

      return {
        dirName: entry.name,
        id: String(Number(match?.[1] ?? "")),
      };
    })
    .sort((a, b) => Number(a.id) - Number(b.id));
}

describe("backlog documentation format", () => {
  const backlogRoot = join(process.cwd(), "docs", "backlog");

  it("defines a single backlog overview and epic summaries", async () => {
    const overviewPath = join(backlogRoot, "README.md");
    const overview = await readFile(overviewPath, "utf8");

    expect(overview).toContain("# LKR Planner Backlog");
    expect(overview).toContain("## Epic Overview");

    const epics = await loadEpicEntries(backlogRoot);
    expect(epics.length).toBeGreaterThan(0);

    const seenEpicIds = new Set<string>();
    for (const epic of epics) {
      const expectedFolderPrefix = `epic-${epic.id.padStart(2, "0")}-`;
      expect(epic.dirName.startsWith(expectedFolderPrefix)).toBe(true);
      expect(seenEpicIds.has(epic.id)).toBe(false);
      seenEpicIds.add(epic.id);

      // Single overview must list every epic and reference its folder path.
      expect(overview).toContain(`### EPIC ${epic.id}:`);
      expect(overview).toContain(`Folder: \`backlog/${epic.dirName}\``);

      // Epic directories should not contain local readmes anymore.
      const epicReadme = join(backlogRoot, epic.dirName, "README.md");
      await expect(readFile(epicReadme, "utf8")).rejects.toThrow();
    }
  });

  it("defines unique BL identifiers with simplified story format", async () => {
    const epics = await loadEpicEntries(backlogRoot);
    const seenBlIds = new Set<string>();

    for (const epic of epics) {
      const epicPath = join(backlogRoot, epic.dirName);
      const files = (await readdir(epicPath, { withFileTypes: true })).filter(
        (entry) => entry.isFile() && entry.name.endsWith(".md"),
      );

      const itemFiles = files
        .map((entry) => entry.name)
        .filter((name) => name !== "README.md")
        .sort();

      expect(itemFiles.length).toBeGreaterThan(0);

      for (const fileName of itemFiles) {
        expect(fileName).toMatch(/^bl-\d{3}-[a-z0-9-]+\.md$/);

        const itemPath = join(epicPath, fileName);
        const content = await readFile(itemPath, "utf8");

        const titleMatch = content.match(/^# BL-(\d{3}): [^\n]+/m);
        expect(titleMatch).not.toBeNull();

        const blId = titleMatch?.[1] ?? "";
        const expectedPrefix = `bl-${blId}-`;
        expect(fileName.startsWith(expectedPrefix)).toBe(true);
        expect(seenBlIds.has(blId)).toBe(false);
        seenBlIds.add(blId);

        expect(content).toContain("## Scope");
        expect(content).toContain("## Acceptance Criteria");
        expect(content).toContain("## Tests (write first)");

        // Top story metadata block is no longer allowed.
        expect(content).not.toMatch(/^- Epic:/m);
        expect(content).not.toMatch(/^- Priority:/m);
        expect(content).not.toMatch(/^- Effort:/m);
        expect(content).not.toMatch(/^- Status:/m);
      }
    }
  });
});
