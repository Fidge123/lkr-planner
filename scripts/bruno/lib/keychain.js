const { execFileSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const localStorePath = path.join(
  os.homedir(),
  "Library/Application Support/de.richter-labor.planner/local-store.json",
);

function readKeychainJson(service, account) {
  const secret = execFileSync(
    "security",
    ["find-generic-password", "-s", service, "-a", account, "-w"],
    { encoding: "utf8" },
  );
  return JSON.parse(secret.trim());
}

function readApiEndpoints() {
  return JSON.parse(fs.readFileSync(localStorePath, "utf8")).apiEndpoints;
}

function usesKeychain(bru) {
  return bru.getEnvVar("credentialSource") === "keychain";
}

module.exports = { readApiEndpoints, readKeychainJson, usesKeychain };
