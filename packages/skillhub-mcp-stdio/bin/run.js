#!/usr/bin/env node
"use strict";

const { spawn } = require("child_process");
const path = require("path");
const os = require("os");

const PLATFORMS = {
  "darwin-arm64": "skillhub-mcp-stdio-darwin-arm64",
  "darwin-x64": "skillhub-mcp-stdio-darwin-x64",
  "win32-x64": "skillhub-mcp-stdio-win32-x64",
};

const key = `${os.platform()}-${os.arch()}`;
const pkg = PLATFORMS[key];

if (!pkg) {
  console.error(
    `skillhub-mcp-stdio: unsupported platform ${key}\n` +
      `Supported: ${Object.keys(PLATFORMS).join(", ")}`
  );
  process.exit(1);
}

let binPath;
try {
  const pkgDir = path.dirname(require.resolve(`${pkg}/package.json`));
  const ext = os.platform() === "win32" ? ".exe" : "";
  binPath = path.join(pkgDir, "bin", `skillhub-mcp-stdio${ext}`);
} catch {
  console.error(
    `skillhub-mcp-stdio: could not find package "${pkg}".\n` +
      `Try reinstalling: npm install -g skillhub-mcp-stdio`
  );
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), { stdio: "inherit" });

for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
  process.on(signal, () => child.kill(signal));
}

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});
