import { readdirSync, realpathSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);

export function checkRuntimeSymlinks(workerRoot) {
  const canonicalRoot = realpathSync(workerRoot);
  let checkedLinks = 0;

  visit(workerRoot);
  return checkedLinks;

  function visit(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const entryPath = path.join(directory, entry.name);
      if (entry.isSymbolicLink()) {
        const canonicalTarget = canonicalizeLink(entryPath);
        if (!isWithin(canonicalRoot, canonicalTarget)) {
          throw new Error(
            `Symbolic link escapes compiler worker: ${entryPath} -> ${canonicalTarget}`,
          );
        }
        checkedLinks++;
      } else if (entry.isDirectory()) {
        visit(entryPath);
      }
    }
  }
}

function canonicalizeLink(linkPath) {
  try {
    return realpathSync(linkPath);
  } catch (error) {
    throw new Error(`Cannot canonicalize compiler worker symbolic link: ${linkPath}`, {
      cause: error,
    });
  }
}

function isWithin(root, target) {
  const relative = path.relative(root, target);
  if (relative === "") {
    return true;
  }
  if (path.isAbsolute(relative)) {
    return false;
  }
  return relative !== ".." && !relative.startsWith(`..${path.sep}`);
}

if (path.resolve(process.argv[1] ?? "") === scriptPath) {
  try {
    const workerRootArgument = process.argv[2];
    if (!workerRootArgument) {
      throw new Error("Usage: check-runtime-symlinks.mjs <worker-root>");
    }
    const workerRoot = path.resolve(workerRootArgument);
    const checkedLinks = checkRuntimeSymlinks(workerRoot);
    console.log(`Compiler worker symlinks checked: ${checkedLinks}`);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
