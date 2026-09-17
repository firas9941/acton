import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { checkRuntimeSymlinks } from "./check-runtime-symlinks.mjs";

test("requires the compiler worker root as a CLI argument", () => {
  const script = fileURLToPath(new URL("./check-runtime-symlinks.mjs", import.meta.url));
  const result = spawnSync(process.execPath, [script], { encoding: "utf8" });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Usage: check-runtime-symlinks\.mjs <worker-root>/);
});

test("installed compiler worker symlinks stay inside its canonical root", () => {
  const workerRoot = fileURLToPath(new URL("../", import.meta.url));
  assert.doesNotThrow(() => checkRuntimeSymlinks(workerRoot));
});

test("accepts symbolic links whose canonical targets stay inside the worker", (t) => {
  const { worker, write } = fixture(t);
  const target = write("packages/compiler/index.js", "export {};\n");
  const link = path.join(worker, "node_modules", ".bin", "compiler");
  mkdirSync(path.dirname(link), { recursive: true });
  symlinkSync(path.relative(path.dirname(link), target), link);

  assert.equal(checkRuntimeSymlinks(worker), 1);
});

test("rejects symbolic links whose canonical targets escape the worker", (t) => {
  const { root, worker } = fixture(t);
  const secret = path.join(root, "secret.txt");
  writeFileSync(secret, "secret");
  const link = path.join(worker, "secret.txt");
  symlinkSync(secret, link);

  assert.throws(
    () => checkRuntimeSymlinks(worker),
    /Symbolic link escapes compiler worker/,
  );
});

test("rejects broken symbolic links", (t) => {
  const { worker } = fixture(t);
  symlinkSync("missing.txt", path.join(worker, "broken.txt"));

  assert.throws(
    () => checkRuntimeSymlinks(worker),
    /Cannot canonicalize compiler worker symbolic link/,
  );
});

function fixture(t) {
  const root = mkdtempSync(path.join(tmpdir(), "verifier-runtime-symlinks-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const worker = path.join(root, "worker");
  mkdirSync(worker);

  return {
    root,
    worker,
    write(filename, content) {
      const file = path.join(worker, filename);
      mkdirSync(path.dirname(file), { recursive: true });
      writeFileSync(file, content);
      return file;
    },
  };
}
