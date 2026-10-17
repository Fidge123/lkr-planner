const { execFileSync } = require("child_process");

function readKeychainJson(service, account) {
  const secret = execFileSync(
    "security",
    ["find-generic-password", "-s", service, "-a", account, "-w"],
    { encoding: "utf8" },
  );
  return JSON.parse(secret.trim());
}

function usesKeychain(bru) {
  return bru.getEnvVar("credentialSource") === "keychain";
}

module.exports = { readKeychainJson, usesKeychain };
