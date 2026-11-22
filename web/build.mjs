// Builds the files the playground needs:  node web/build.mjs
//   web/lz.wasm        the system, compiled for WASI
//   web/lz-lsp.wasm    the language server, compiled for WASI (if it builds)
//   web/examples.json  the example tapes, with the page each types
import { execSync } from "node:child_process";
import { copyFileSync, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

// (fileURLToPath: the directory's name has an en dash in it.)
const root = fileURLToPath(new URL("..", import.meta.url));
execSync("cargo build --release --target wasm32-wasip1 -p laning-zierler", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-wasip1/release/lz.wasm`, `${root}web/lz.wasm`);
let lsp = false;
try {
  execSync("cargo build --release --target wasm32-wasip1 -p lz-lsp", { cwd: root, stdio: "inherit" });
  const built = `${root}target/wasm32-wasip1/release/lz-lsp.wasm`;
  if (existsSync(built)) {
    copyFileSync(built, `${root}web/lz-lsp.wasm`);
    lsp = true;
  }
} catch {
  console.warn("lz-lsp did not build: the editor will work without it");
}

// Shown first, in this order; the rest follow alphabetically.
const first = [
  "1954-first-equations",
  "1954-division",
  "1954-cosine-series",
  "1954-differential-equations",
  "1954-innocuous-change",
  "1954-zero",
  "1976-knuth-tpk",
];

const names = readdirSync(`${root}examples`).filter((f) => f.endsWith(".lzr")).map((f) => f.slice(0, -4));
names.sort((a, b) => {
  const [i, j] = [first.indexOf(a), first.indexOf(b)];
  if (i !== -1 || j !== -1) return (i === -1 ? 99 : i) - (j === -1 ? 99 : j);
  return a.localeCompare(b);
});
const examples = names.map((name) => {
  const tape = readFileSync(`${root}examples/${name}.lzr`, "utf8");
  const out = existsSync(`${root}examples/${name}.out`) ? readFileSync(`${root}examples/${name}.out`, "utf8") : null;
  // The notes at the top of the tape say what it is.
  const notes = [];
  for (const line of tape.split("\n")) {
    if (!line.startsWith("#")) break;
    notes.push(line.replace(/^#\s?/, ""));
  }
  const options = /^# options:(.*)$/m.exec(tape);
  const settings = {};
  if (options) {
    if (/--zero\s+plus/.test(options[1])) settings.zero = "plus";
    if (/--temporary-rules/.test(options[1])) settings.rules = true;
  }
  return { name, tape, out, about: notes.join("\n").trim(), settings };
});
writeFileSync(`${root}web/examples.json`, JSON.stringify(examples));
console.log(`web/lz.wasm, ${lsp ? "web/lz-lsp.wasm, " : ""}web/examples.json (${examples.length} tapes) written`);
