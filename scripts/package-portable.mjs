import { copyFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const exeName = "Yuli Ledger.exe";
const src = join(root, "src-tauri", "target", "release", exeName);
const destDir = join(root, "release", "portable");
const dest = join(destDir, exeName);

if (!existsSync(src)) {
  console.error(`Missing ${src}. Run npm run tauri -- build first.`);
  process.exit(1);
}

mkdirSync(destDir, { recursive: true });
try {
  copyFileSync(src, dest);
  console.log(`Portable exe: ${dest}`);
} catch (err) {
  const code = err && typeof err === "object" && "code" in err ? err.code : "";
  if (code !== "EBUSY" && code !== "EPERM") {
    throw err;
  }
  const fallback = join(destDir, "Yuli Ledger.new.exe");
  copyFileSync(src, fallback);
  console.warn(`Could not overwrite ${dest} (file in use). Wrote ${fallback} instead.`);
}
console.log("Database remains at %LOCALAPPDATA%\\YuliLedger\\ledger.sqlite");
