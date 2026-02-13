import { readFile, writeFile } from "node:fs/promises";

interface TauriConfig {
  version: string;
}

export function getEpochMilliseconds(): string {
  return String(Date.now());
}

export function createStampedReleaseVersion(baseVersion: string): string {
  const epochMilliseconds = getEpochMilliseconds();

  const stamped = `${baseVersion}-main.${epochMilliseconds}`;

  return stamped;
}

export async function stampTauriConfig(configPath: string): Promise<string> {
  const raw = await readFile(configPath, "utf8");
  const config = JSON.parse(raw) as TauriConfig;
  const stampedVersion = createStampedReleaseVersion(config.version);

  config.version = stampedVersion;
  await writeFile(configPath, `${JSON.stringify(config, null, 2)}\n`, "utf8");

  return stampedVersion;
}

async function main() {
  const configPath = "src-tauri/tauri.conf.json";
  const stampedVersion = await stampTauriConfig(configPath);

  console.log(`Stamped release version: ${stampedVersion}`);

  if (process.env.GITHUB_OUTPUT) {
    await writeFile(
      process.env.GITHUB_OUTPUT,
      `stamped_version=${stampedVersion}\n`,
      { encoding: "utf8", flag: "a" },
    );
  }
}

if (import.meta.main) {
  main().catch((error) => {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`Version stamping failed: ${message}`);
    process.exit(1);
  });
}
