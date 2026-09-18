import { existsSync, readdirSync } from "node:fs";
import { delimiter, dirname, join, resolve } from "node:path";
import { spawn } from "node:child_process";
import { platform } from "node:os";
import { fileURLToPath } from "node:url";

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function hasLibclang(directory) {
  if (!directory || !existsSync(directory)) return false;

  try {
    return readdirSync(directory).some((entry) => {
      const name = entry.toLowerCase();
      return (
        name === "libclang.dll" ||
        name === "clang.dll" ||
        name === "libclang.dylib" ||
        name.startsWith("libclang.so")
      );
    });
  } catch {
    return false;
  }
}

function asDirectory(value) {
  if (!value) return undefined;
  const candidate = resolve(value);
  if (hasLibclang(candidate)) return candidate;

  const lowerName = candidate.toLowerCase();
  if (
    lowerName.endsWith(".dll") ||
    lowerName.endsWith(".dylib") ||
    lowerName.includes("libclang.so")
  ) {
    const directory = dirname(candidate);
    return hasLibclang(directory) ? directory : undefined;
  }

  return undefined;
}

function addCandidate(candidates, value) {
  const directory = asDirectory(value);
  if (directory && !candidates.includes(directory)) candidates.push(directory);
}

function addPythonClangCandidates(candidates, pythonRoot) {
  if (!pythonRoot || !existsSync(pythonRoot)) return;

  let versions;
  try {
    versions = readdirSync(pythonRoot, { withFileTypes: true })
      .filter((entry) => entry.isDirectory() && /^Python\d+$/i.test(entry.name))
      .map((entry) => entry.name);
  } catch {
    return;
  }

  for (const version of versions) {
    addCandidate(
      candidates,
      join(pythonRoot, version, "site-packages", "clang", "native"),
    );
    addCandidate(
      candidates,
      join(pythonRoot, version, "Lib", "site-packages", "clang", "native"),
    );
  }
}

function findLibclang() {
  const candidates = [];
  const configured = process.env.LIBCLANG_PATH?.split(delimiter) ?? [];
  configured.forEach((value) => addCandidate(candidates, value));

  const pathEntries = process.env.PATH?.split(delimiter) ?? [];
  pathEntries.forEach((value) => addCandidate(candidates, value));

  if (platform() === "win32") {
    addCandidate(candidates, "C:/Program Files/LLVM/bin");
    addCandidate(candidates, "C:/Program Files (x86)/LLVM/bin");
    addCandidate(
      candidates,
      join(process.env.LOCALAPPDATA ?? "", "Programs", "LLVM", "bin"),
    );
    addPythonClangCandidates(
      candidates,
      process.env.APPDATA ? join(process.env.APPDATA, "Python") : undefined,
    );
    addPythonClangCandidates(
      candidates,
      process.env.LOCALAPPDATA
        ? join(process.env.LOCALAPPDATA, "Python")
        : undefined,
    );
  } else {
    addCandidate(candidates, "/usr/lib/llvm/lib");
    addCandidate(candidates, "/usr/lib/x86_64-linux-gnu");
    addCandidate(candidates, "/usr/lib/aarch64-linux-gnu");
    addCandidate(candidates, "/usr/local/opt/llvm/lib");
    addCandidate(candidates, "/opt/homebrew/opt/llvm/lib");
  }

  return candidates[0];
}

function findNativeCmake() {
  if (platform() !== "win32") return undefined;

  const candidates = [
    "C:/Program Files/CMake/bin/cmake.exe",
    "C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin/cmake.exe",
    "C:/Program Files/Microsoft Visual Studio/2022/BuildTools/Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin/cmake.exe",
  ];

  return candidates.find((candidate) => existsSync(candidate));
}

const libclangPath = findLibclang();
if (libclangPath) {
  process.env.LIBCLANG_PATH = libclangPath;
  console.log(`[bloody-level] Using libclang from ${libclangPath}`);
} else {
  console.warn(
    "[bloody-level] libclang was not found. Install LLVM or Python clang bindings before building the native app.",
  );
}

const nativeCmake = findNativeCmake();
if (!process.env.CMAKE && nativeCmake) {
  process.env.CMAKE = nativeCmake;
  console.log(`[bloody-level] Using native CMake from ${nativeCmake}`);
}

const cli = join(
  repositoryRoot,
  "node_modules",
  "@tauri-apps",
  "cli",
  "tauri.js",
);
const child = spawn(process.execPath, [cli, ...process.argv.slice(2)], {
  cwd: repositoryRoot,
  env: process.env,
  stdio: "inherit",
});

child.on("error", (error) => {
  console.error(`[bloody-level] Could not start Tauri CLI: ${error.message}`);
  process.exitCode = 1;
});

child.on("exit", (code, signal) => {
  if (signal) {
    console.error(`[bloody-level] Tauri CLI exited from signal ${signal}`);
    process.exitCode = 1;
  } else {
    process.exitCode = code ?? 1;
  }
});
