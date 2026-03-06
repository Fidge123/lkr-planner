import { spawnSync } from "node:child_process";
import { copyFile, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

interface GlyphBounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  width: number;
  height: number;
}

export interface GlyphOutline {
  fontName: string;
  pathData: string;
  bounds: GlyphBounds;
}

interface CommandResult {
  stdout: string;
  stderr: string;
}

const iconViewBoxSize = 1024;
const backgroundPathData =
  "M 100 724 L 100 300 C 100 185.001 170 99.995 300.026 100 L 724 99.995 C 854 100 924 185.001 924 300 L 924 724 C 924 839.001 854 923.995 724 924 L 300.026 924 C 170 924 100 839.001 100 724 Z";
const glyphTargetHeight = 448;
const glyphCenterX = 512;
const glyphCenterY = 512;
const projectRoot = resolve(import.meta.dir, "..");
const iconsDir = join(projectRoot, "src-tauri", "icons");
const swiftHelperPath = join(import.meta.dir, "extract-app-icon-glyph.swift");
const iconSvgPath = join(iconsDir, "icon.svg");
const iconPngPath = join(iconsDir, "icon.png");
const iconIcoPath = join(iconsDir, "icon.ico");
const iconIcnsPath = join(iconsDir, "icon.icns");

export const validationRasterSizes = [16, 32, 64, 128, 256, 512, 1024];
const icoValidationRasterSizes = [16, 24, 32, 48, 64, 256];
const generatedValidationRasterSizes = Array.from(
  new Set([...validationRasterSizes, ...icoValidationRasterSizes]),
).sort((left, right) => left - right);

function formatNumber(value: number): string {
  const rounded = Math.round(value * 1000) / 1000;
  if (Object.is(rounded, -0)) {
    return "0";
  }

  return `${rounded}`.replace(/\.0+$/, "").replace(/(\.\d*?)0+$/, "$1");
}

function createGlyphTransform(glyph: GlyphOutline): string {
  const scale = glyphTargetHeight / glyph.bounds.height;
  const glyphCenterSourceX = (glyph.bounds.minX + glyph.bounds.maxX) / 2;
  const glyphCenterSourceY = (glyph.bounds.minY + glyph.bounds.maxY) / 2;
  const translateX = glyphCenterX - glyphCenterSourceX * scale;
  const translateY = glyphCenterY + glyphCenterSourceY * scale;

  return `translate(${formatNumber(translateX)} ${formatNumber(translateY)}) scale(${formatNumber(scale)} ${formatNumber(-scale)})`;
}

export function buildIconSvg(glyph: GlyphOutline): string {
  const transform = createGlyphTransform(glyph);

  return `<?xml version="1.0" encoding="utf-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${iconViewBoxSize} ${iconViewBoxSize}" width="${iconViewBoxSize}" height="${iconViewBoxSize}">
  <title>LKR Planner App Icon</title>
  <path fill="#4A90E2" d="${backgroundPathData}" />
  <path fill="#FFFFFF" data-font="${glyph.fontName}" d="${glyph.pathData}" transform="${transform}" />
</svg>
`;
}

function runCommand(
  command: string,
  args: string[],
  {
    cwd = projectRoot,
    env = {},
    allowedExitCodes = [0],
  }: {
    cwd?: string;
    env?: NodeJS.ProcessEnv;
    allowedExitCodes?: number[];
  } = {},
): CommandResult {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    env: {
      ...process.env,
      ...env,
    },
  });

  if (result.error) {
    throw result.error;
  }

  const exitCode = result.status ?? 0;
  if (!allowedExitCodes.includes(exitCode)) {
    const stdout = result.stdout.trim();
    const stderr = result.stderr.trim();
    throw new Error(
      [
        `Command failed: ${command} ${args.join(" ")}`,
        stdout.length > 0 ? `stdout:\n${stdout}` : "",
        stderr.length > 0 ? `stderr:\n${stderr}` : "",
      ]
        .filter(Boolean)
        .join("\n\n"),
    );
  }

  return {
    stderr: result.stderr.trim(),
    stdout: result.stdout.trim(),
  };
}

function loadGlyphOutline(
  fontName: string,
  glyphCharacter: string,
): GlyphOutline {
  const swiftModuleCachePath = join(tmpdir(), "lkr-planner-swift-module-cache");
  const clangModuleCachePath = join(tmpdir(), "lkr-planner-clang-module-cache");

  runCommand("mkdir", ["-p", swiftModuleCachePath, clangModuleCachePath]);

  const { stdout } = runCommand(
    "swift",
    [swiftHelperPath, fontName, glyphCharacter],
    {
      env: {
        CLANG_MODULE_CACHE_PATH: clangModuleCachePath,
        SWIFT_MODULECACHE_PATH: swiftModuleCachePath,
      },
    },
  );

  return JSON.parse(stdout) as GlyphOutline;
}

async function generateTauriIcons(outputDir: string) {
  runCommand("bun", ["tauri", "icon", iconSvgPath, "-o", outputDir]);
}

async function generateValidationRasters(outputDir: string) {
  runCommand("bun", [
    "tauri",
    "icon",
    iconSvgPath,
    "-o",
    outputDir,
    ...generatedValidationRasterSizes.flatMap((size) => ["-p", `${size}`]),
  ]);
}

function parseCompareMetric(output: string): number {
  const metricText = output.trim();
  if (metricText === "0" || metricText === "0 (0)") {
    return 0;
  }

  const match = metricText.match(/^([0-9]+(?:\.[0-9]+)?)(?:\s+\([^)]+\))?$/);
  if (!match) {
    throw new Error(`Unable to parse compare metric from output: ${output}`);
  }

  return Number.parseFloat(match[1]);
}

function compareImages(
  expectedPath: string,
  actualPath: string,
  label: string,
) {
  const result = runCommand(
    "magick",
    ["compare", "-metric", "AE", expectedPath, actualPath, "null:"],
    { allowedExitCodes: [0, 1] },
  );
  const metric = parseCompareMetric(result.stderr || result.stdout);

  if (metric !== 0) {
    throw new Error(`${label} differs from the SVG render (${metric} pixels).`);
  }
}

function extractIcoFrameBySize(
  icoPath: string,
  size: number,
  outputPath: string,
) {
  const { stdout } = runCommand("magick", [
    "identify",
    "-format",
    "%p %w %h\n",
    icoPath,
  ]);

  const frameLine = stdout
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .find((line) => {
      const [frameIndexText, widthText, heightText] = line.split(" ");
      return (
        Number.parseInt(frameIndexText, 10) >= 0 &&
        Number.parseInt(widthText, 10) === size &&
        Number.parseInt(heightText, 10) === size
      );
    });

  if (!frameLine) {
    throw new Error(`Unable to locate a ${size}px frame in ${icoPath}.`);
  }

  const frameIndex = frameLine.split(" ")[0];
  runCommand("magick", [`${icoPath}[${frameIndex}]`, outputPath]);
}

function createValidationRasterMap(outputDir: string): Map<number, string> {
  return new Map(
    generatedValidationRasterSizes.map((size) => [
      size,
      join(outputDir, `${size}x${size}.png`),
    ]),
  );
}

async function validateOutputs(
  tempDir: string,
  rasterBySize: Map<number, string>,
) {
  compareImages(rasterBySize.get(512) ?? "", iconPngPath, "icon.png");

  const extractedIcnsDir = join(tempDir, "icon.icns.iconset");
  runCommand("iconutil", [
    "-c",
    "iconset",
    iconIcnsPath,
    "-o",
    extractedIcnsDir,
  ]);

  const icnsFileBySize = new Map<number, string>([
    [16, join(extractedIcnsDir, "icon_16x16.png")],
    [32, join(extractedIcnsDir, "icon_32x32.png")],
    [64, join(extractedIcnsDir, "icon_32x32@2x.png")],
    [128, join(extractedIcnsDir, "icon_128x128.png")],
    [256, join(extractedIcnsDir, "icon_256x256.png")],
    [512, join(extractedIcnsDir, "icon_512x512.png")],
    [1024, join(extractedIcnsDir, "icon_512x512@2x.png")],
  ]);

  for (const size of validationRasterSizes) {
    const svgRasterPath = rasterBySize.get(size);
    const icnsRasterPath = icnsFileBySize.get(size);
    if (!svgRasterPath || !icnsRasterPath) {
      throw new Error(`Missing validation raster for ${size}px.`);
    }

    compareImages(svgRasterPath, icnsRasterPath, `icon.icns ${size}px`);
  }

  for (const size of icoValidationRasterSizes) {
    const svgRasterPath = rasterBySize.get(size);
    if (!svgRasterPath) {
      throw new Error(`Missing SVG raster for ${size}px.`);
    }

    const extractedIcoPath = join(tempDir, `icon-${size}.ico.png`);
    extractIcoFrameBySize(iconIcoPath, size, extractedIcoPath);
    compareImages(svgRasterPath, extractedIcoPath, `icon.ico ${size}px`);
  }
}

async function main() {
  const tempDir = await mkdtemp(join(tmpdir(), "lkr-planner-icon-"));

  try {
    const glyph = loadGlyphOutline("HelveticaNeue-Bold", "R");
    const svg = buildIconSvg(glyph);
    await writeFile(iconSvgPath, svg, "utf8");

    const generatedIconsDir = join(tempDir, "generated");
    const validationRastersDir = join(tempDir, "validation");
    await generateTauriIcons(generatedIconsDir);
    await generateValidationRasters(validationRastersDir);

    await copyFile(join(generatedIconsDir, "icon.png"), iconPngPath);
    await copyFile(join(generatedIconsDir, "icon.ico"), iconIcoPath);
    await copyFile(join(generatedIconsDir, "icon.icns"), iconIcnsPath);

    const rasterBySize = createValidationRasterMap(validationRastersDir);
    await validateOutputs(tempDir, rasterBySize);

    console.log(
      `Regenerated icon.svg, icon.png, icon.ico and icon.icns with ${glyph.fontName}. Validation passed for ${validationRasterSizes.join(", ")} px.`,
    );
  } finally {
    await rm(tempDir, { force: true, recursive: true });
  }
}

if (import.meta.main) {
  main().catch((error) => {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`Icon regeneration failed: ${message}`);
    process.exit(1);
  });
}
