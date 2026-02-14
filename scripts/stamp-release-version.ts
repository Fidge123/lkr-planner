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

function stampTauriConfigContent(
  rawConfig: string,
  stampedVersion: string,
): string {
  const config = JSON.parse(rawConfig) as TauriConfig;
  config.version = stampedVersion;
  return `${JSON.stringify(config, null, 2)}\n`;
}

function stampCargoTomlPackageVersion(
  rawCargoToml: string,
  stampedVersion: string,
): string {
  const lines = rawCargoToml.split(/\r?\n/);
  let inPackageSection = false;
  let replaced = false;

  const stampedLines = lines.map((line) => {
    const trimmed = line.trim();

    if (trimmed.startsWith("[")) {
      inPackageSection = trimmed === "[package]";
      return line;
    }

    if (inPackageSection && !replaced && /^version\s*=\s*".*"$/.test(trimmed)) {
      replaced = true;
      const indentation = line.match(/^\s*/)?.[0] ?? "";
      return `${indentation}version = "${stampedVersion}"`;
    }

    return line;
  });

  if (!replaced) {
    throw new Error(
      "Could not stamp Cargo.toml package version: missing [package].version field.",
    );
  }

  return `${stampedLines.join("\n")}\n`;
}

export async function stampReleaseVersionFiles(
  tauriConfigPath: string,
  cargoTomlPath: string,
): Promise<string> {
  const tauriRaw = await readFile(tauriConfigPath, "utf8");
  const tauriConfig = JSON.parse(tauriRaw) as TauriConfig;
  const stampedVersion = createStampedReleaseVersion(tauriConfig.version);

  const stampedTauriConfig = stampTauriConfigContent(tauriRaw, stampedVersion);
  await writeFile(tauriConfigPath, stampedTauriConfig, "utf8");

  const cargoRaw = await readFile(cargoTomlPath, "utf8");
  const stampedCargoToml = stampCargoTomlPackageVersion(
    cargoRaw,
    stampedVersion,
  );
  await writeFile(cargoTomlPath, stampedCargoToml, "utf8");

  return stampedVersion;
}

async function main() {
  const tauriConfigPath = "src-tauri/tauri.conf.json";
  const cargoTomlPath = "src-tauri/Cargo.toml";
  const stampedVersion = await stampReleaseVersionFiles(
    tauriConfigPath,
    cargoTomlPath,
  );

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
