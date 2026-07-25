import { spawnSync } from "node:child_process";

// CI runners default to UTC, where a value built in local time and one built in
// UTC are indistinguishable, so every bug from mixing the two passes unnoticed.
const timezones = [
  "UTC",
  "Europe/Berlin", // production timezone
  "America/New_York", // negative offset
  "Pacific/Kiritimati", // extreme positive offset
  "Asia/Kolkata", // half-hour offset
  "Pacific/Midway", // extreme negative offset
];

const failed: string[] = [];

for (const timezone of timezones) {
  console.log(`\n=== bun test (TZ=${timezone}) ===`);
  const result = spawnSync("bun", ["test"], {
    stdio: "inherit",
    env: { ...process.env, TZ: timezone },
  });
  if (result.status !== 0) {
    failed.push(timezone);
  }
}

if (failed.length > 0) {
  console.error(`\nFailing timezones: ${failed.join(", ")}`);
  process.exit(1);
}

console.log(`\nAll ${timezones.length} timezones passed.`);
