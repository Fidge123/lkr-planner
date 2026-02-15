import { readFile, writeFile } from "node:fs/promises";
import { parse, stringify, type TomlTable } from "smol-toml";

interface TauriConfig {
  version: string;
}

type CargoToml = {
  package?: {
    version?: string;
  };
} & TomlTable;

export function createStampedReleaseVersion(baseVersion: string): string {
  return `${baseVersion}-main.${String(Date.now())}`;
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
  const parsedToml = parse(rawCargoToml) as CargoToml;

  if (!parsedToml.package?.version) {
    throw new Error("Cargo.toml is missing [package].version field.");
  }

  parsedToml.package.version = stampedVersion;
  return `${stringify(parsedToml)}\n`.replace(/\n+$/, "\n");
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
