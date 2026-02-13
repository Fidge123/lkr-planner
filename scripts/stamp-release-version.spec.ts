import { describe, expect, it, setSystemTime } from "bun:test";
import { createStampedReleaseVersion } from "./stamp-release-version";

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
});
