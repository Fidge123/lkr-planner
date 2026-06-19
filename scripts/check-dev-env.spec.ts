import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const SCRIPT = join(import.meta.dir, "check-dev-env.sh");

let workDir: string;

beforeEach(() => {
  workDir = mkdtempSync(join(tmpdir(), "check-dev-env-"));
});

afterEach(() => {
  rmSync(workDir, { recursive: true, force: true });
});

function runCheck(env: Record<string, string>) {
  const result = Bun.spawnSync(["/bin/sh", SCRIPT], { env });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
  };
}

describe("check-dev-env.sh", () => {
  test("warns for each missing tool and exits 0", () => {
    const emptyBrowsers = join(workDir, "no-browsers");
    mkdirSync(emptyBrowsers);

    const { exitCode, stderr } = runCheck({
      PATH: "",
      HOME: workDir,
      PLAYWRIGHT_BROWSERS_PATH: emptyBrowsers,
    });

    expect(exitCode).toBe(0);
    expect(stderr).toContain("bun");
    expect(stderr).toContain("cargo");
    expect(stderr).toContain("chromium");
  });

  test("stays silent and exits 0 when all tools are present", () => {
    const bin = join(workDir, "bin");
    mkdirSync(bin);
    for (const tool of ["bun", "cargo"]) {
      const path = join(bin, tool);
      writeFileSync(path, "#!/bin/sh\n");
      chmodSync(path, 0o755);
    }

    const browsers = join(workDir, "browsers");
    const chromeDir = join(browsers, "chromium-1234", "chrome-linux");
    mkdirSync(chromeDir, { recursive: true });
    const chrome = join(chromeDir, "chrome");
    writeFileSync(chrome, "#!/bin/sh\n");
    chmodSync(chrome, 0o755);

    const { exitCode, stderr } = runCheck({
      PATH: bin,
      HOME: workDir,
      PLAYWRIGHT_BROWSERS_PATH: browsers,
    });

    expect(exitCode).toBe(0);
    expect(stderr.trim()).toBe("");
  });
});
