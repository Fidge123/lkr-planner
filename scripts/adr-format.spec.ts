import { describe, expect, it } from "bun:test";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

const evaluatedOptionsRequirementStartId = 8;
const evaluatedOptionsError =
  "Context must include an evaluated options list with pros and cons for ADR IDs above 0008.";

function extractSection(content: string, sectionName: string): string | null {
  const escapedSectionName = sectionName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const sectionMatch = content.match(
    new RegExp(`## ${escapedSectionName}\\n\\n([\\s\\S]*?)(?=\\n## |$)`),
  );

  return sectionMatch?.[1] ?? null;
}

function hasEvaluatedOptionsWithProsAndCons(contextSection: string): boolean {
  const evaluatedOptionsSection = contextSection.match(
    /### Evaluated Options\s*\n([\s\S]*?)(?=\n### |\n## |$)/,
  )?.[1];

  if (!evaluatedOptionsSection) {
    return false;
  }

  const optionBlocks = evaluatedOptionsSection.match(
    /- [^\n]+\n(?:[ \t]+- Pros: [^\n]+\n[ \t]+- Cons: [^\n]+|[ \t]+- Cons: [^\n]+\n[ \t]+- Pros: [^\n]+)/g,
  );

  return (optionBlocks?.length ?? 0) > 0;
}

function validateAdrContent(fileName: string, content: string): string[] {
  const errors: string[] = [];

  if (!/^- Status: [^\n]+$/m.test(content)) {
    errors.push("Missing or invalid ADR status metadata line.");
  }

  if (!/^- Date: \d{4}-\d{2}-\d{2}$/m.test(content)) {
    errors.push("Missing or invalid ADR date metadata line.");
  }

  if (!content.includes("## Context")) {
    errors.push("Missing required section: Context.");
  }

  if (!content.includes("## Decision")) {
    errors.push("Missing required section: Decision.");
  }

  if (!content.includes("## Consequences")) {
    errors.push("Missing required section: Consequences.");
  }

  const adrId = Number.parseInt(fileName.slice(0, 4), 10);
  if (adrId > evaluatedOptionsRequirementStartId) {
    const contextSection = extractSection(content, "Context");
    if (
      contextSection === null ||
      !hasEvaluatedOptionsWithProsAndCons(contextSection)
    ) {
      errors.push(evaluatedOptionsError);
    }
  }

  return errors;
}

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
      const errors = validateAdrContent(fileName, content);
      expect(errors).toEqual([]);
    }
  });
});
