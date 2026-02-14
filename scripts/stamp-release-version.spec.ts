import { describe, expect, it, setSystemTime } from "bun:test";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  createStampedReleaseVersion,
  stampReleaseVersionFiles,
} from "./stamp-release-version";

describe("stamp release version", () => {
  it("creates epoch-millisecond prerelease version in expected format", () => {
    setSystemTime(new Date("2025-02-13T15:30:45.000Z"));
    const version = createStampedReleaseVersion("0.1.0");

    expect(version).toBe("0.1.0-main.1739460645000");
  });

  it("creates unique and chronologically sortable versions", () => {
    setSystemTime(new Date("2025-02-13T15:30:45.000Z"));
    const earlier = createStampedReleaseVersion("0.1.0");
    setSystemTime(new Date("2025-02-13T15:30:45.001Z"));
    const later = createStampedReleaseVersion("0.1.0");

    expect(earlier).not.toBe(later);
    expect(earlier < later).toBe(true);
  });

  it("stamps tauri config and cargo package version with the same stamped version", async () => {
    setSystemTime(new Date("2025-02-13T15:30:45.000Z"));

    const testDir = await mkdtemp(join(tmpdir(), "lkr-planner-stamp-test-"));
    const tauriConfigPath = join(testDir, "tauri.conf.json");
    const cargoTomlPath = join(testDir, "Cargo.toml");

    await writeFile(
      tauriConfigPath,
      `${JSON.stringify({ version: "0.1.0" }, null, 2)}\n`,
      "utf8",
    );
    await writeFile(
      cargoTomlPath,
      `[package]
name = "lkr-planner"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
`,
      "utf8",
    );

    const stampedVersion = await stampReleaseVersionFiles(
      tauriConfigPath,
      cargoTomlPath,
    );

    expect(stampedVersion).toBe("0.1.0-main.1739460645000");

    const stampedTauriConfig = JSON.parse(
      await readFile(tauriConfigPath, "utf8"),
    ) as { version: string };
    expect(stampedTauriConfig.version).toBe(stampedVersion);

    const stampedCargoToml = await readFile(cargoTomlPath, "utf8");
    expect(stampedCargoToml).toContain(`version = "${stampedVersion}"`);
    expect(stampedCargoToml).toContain(
      'serde = { version = "1", features = ["derive"] }',
    );
  });
});
