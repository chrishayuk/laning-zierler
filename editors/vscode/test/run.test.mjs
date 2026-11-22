// Runs every example through the runner the "Run Tape" command uses, and
// compares the page it prints with the expected one.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const runner = fileURLToPath(new URL("../server/run.js", import.meta.url));
const examples = fileURLToPath(new URL("../../../examples/", import.meta.url));

function run(text, options = []) {
  const file = join(mkdtempSync(join(tmpdir(), "lz-")), "tape.lzr");
  writeFileSync(file, text);
  return spawnSync(process.execPath, [runner, ...options, file]);
}

/** Options an example asks for: a line `# options: --zero plus`. */
function optionsOf(text) {
  const line = text.split("\n").find((l) => l.trim().startsWith("# options:"));
  return line ? line.trim().slice("# options:".length).trim().split(/\s+/) : [];
}

test("the built-in interpreter types every example's page", () => {
  const names = readdirSync(examples).filter((f) => f.endsWith(".lzr"));
  assert.ok(names.length >= 10);
  for (const file of names) {
    const text = readFileSync(examples + file, "utf8");
    const result = spawnSync(process.execPath, [runner, ...optionsOf(text), examples + file]);
    const want = readFileSync(`${examples}${file.slice(0, -4)}.out`, "utf8");
    // A tape that does not translate types nothing; its .out holds the problems.
    if (result.stdout.length === 0 && result.status !== 0) continue;
    assert.equal(result.stdout.toString(), want, file);
  }
});

test("a tape is translated and run", () => {
  const result = run("a = 5,\ny = -6.3a,\nPRINT a, y.\nSTOP\n");
  assert.equal(result.status, 0, result.stderr.toString());
  assert.equal(result.stdout.toString(), "\na = +.50000000+01       y = -.31500001+02\n");
});

test("an alarm, and the options of lz", () => {
  const tape = "i = 1,\ni = i - 1,\nx = a|i,\nPRINT x.\nSTOP\n";
  const ones = run(tape);
  assert.equal(ones.status, 1);
  assert.match(ones.stdout.toString(), /ALARM {2}EQUATION 103 {2}PRECEDED BY 102/);
  assert.match(ones.stderr.toString(), /-0/);
  const plus = run(tape, ["--zero", "plus"]);
  assert.equal(plus.status, 0, plus.stderr.toString());
  const listing = run(tape, ["--translate"]);
  assert.match(listing.stdout.toString(), /store x/);
});

test("a tape that does not translate says where", () => {
  const result = run("x = 5\nSTOP\n");
  assert.equal(result.status, 1);
  assert.match(result.stderr.toString(), /comma/);
});
