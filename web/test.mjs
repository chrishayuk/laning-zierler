// Feeds every example through the WebAssembly build and the page's own code,
// and compares with the .out files and with the native build. Then asks the
// language server, through the worker the page's editor uses, about the same
// examples.
//   node web/test.mjs
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { check as lzCheck, compareZeros, feed, trace, translation } from "./tape.js";
import { localTokens } from "./editor.js";

// (fileURLToPath: the directory's name has an en dash in it.)
const root = fileURLToPath(new URL("..", import.meta.url));
const module = new WebAssembly.Module(readFileSync(root + "web/lz.wasm"));
const native = `${root}target/release/lz`;
let failed = 0;
function check(what, ok, detail = "") {
  if (ok) return;
  failed++;
  console.log(`FAIL ${what} ${detail}`);
}

// The settings an example asks for in a `# options:` line.
function settingsOf(tape) {
  const m = /^# options:(.*)$/m.exec(tape);
  return { zero: m && /--zero\s+plus/.test(m[1]) ? "plus" : "ones", rules: !!(m && /--temporary-rules/.test(m[1])) };
}

// Every example: the page is the .out file, and the native build agrees.
let examples = 0;
for (const file of readdirSync(root + "examples").filter((f) => f.endsWith(".lzr")).sort()) {
  const name = file.slice(0, -4);
  const tape = readFileSync(`${root}examples/${file}`, "utf8");
  const settings = settingsOf(tape);
  const result = feed(module, tape, settings);
  const out = `${root}examples/${name}.out`;
  if (existsSync(out)) {
    const expected = readFileSync(out, "utf8");
    // An example that does not translate has its problems in the .out file.
    const got = result.translated ? result.page : result.problems.map((p) => `${p.line}:${p.column}: error: ${p.message}`).join("\n") + "\n";
    check(`example ${name}`, got === expected, `\n--- page\n${got}\n--- .out\n${expected}`);
  }
  if (existsSync(native)) {
    const args = ["--zero", settings.zero, ...(settings.rules ? ["--temporary-rules"] : [])];
    let there;
    try {
      there = execFileSync(native, args, { input: tape, encoding: "utf8", stdio: ["pipe", "pipe", "ignore"] });
    } catch (e) {
      there = e.stdout;
    }
    check(`example ${name} native`, there === result.page);
  }
  // The translation lists every numbered equation.
  const t = translation(module, tape, settings);
  check(`translation ${name}`, !result.translated || /^\s*\d+  /m.test(t.text), t.text.slice(0, 200));
  examples++;
}

// The page's readings of what lz says.
const tpk = readFileSync(`${root}examples/1976-knuth-tpk.lzr`, "utf8");
const ran = feed(module, tpk);
check("an alarm is read", ran.alarm && ran.alarm.includes("-0"), JSON.stringify(ran.alarm));
check("the costs are read", ran.stats.some((r) => r.label === "drum variable accesses" && r.value > 0), JSON.stringify(ran.stats));
const both = compareZeros(module, tpk);
check("the two zeros differ on TPK", both.ones.alarm && !both.plus.alarm && both.differ.length > 0, JSON.stringify(both.differ));
const rules = feed(module, tpk, { rules: true });
check("the rules of January 1954", !rules.translated && /not yet in operation/.test(rules.problems[0]?.message), JSON.stringify(rules.problems));
const bad = feed(module, "a = (1,\nSTOP");
check("a tape that does not translate", !bad.translated && bad.problems[0]?.line === 1, JSON.stringify(bad));
const traced = trace(module, readFileSync(`${root}examples/1954-differential-equations.lzr`, "utf8"));
check("trace", traced.lines.some((l) => /pass 4/.test(l)) && traced.lines.some((l) => /CP 2/.test(l)), traced.lines.slice(0, 5).join("\n"));
const endless = feed(module, "1 x = x + 1,\nSP 1,\nSTOP", { limit: 10000 });
check("a loop that does not end is cut off", endless.limit, JSON.stringify(endless));
const warned = lzCheck(module, "n = m - 1,\nm = 1,\nSTOP");
check("lz --check", warned.some((p) => p.severity === "warning" && p.line === 1), JSON.stringify(warned));

// The editor's own colouring, used when the language server gives none.
const kinds = (text) => localTokens(text).map(([a, b, k]) => `${text.slice(a, b)}:${k}`);
check("local tokens", JSON.stringify(kinds("1 y = F^2(x|5)^2, # n")) ===
  JSON.stringify(["1:label", "y:variable", "=:operator", "F^2:function", "(:operator", "x:variable", "|5:index", "):operator", "^2:exponent", ",:operator", "# n:comment"]),
  JSON.stringify(kinds("1 y = F^2(x|5)^2, # n")));
check("local tokens: D and PRINT", JSON.stringify(kinds("D y|1 = a, PRINT a.")).includes('"D:keyword"') && JSON.stringify(kinds("PRINT a.")).includes('"PRINT:keyword"'));

// The language server, as the editor asks it.
let asked = 0;
let lspNote = "no web/lz-lsp.wasm";
if (existsSync(root + "web/lz-lsp.wasm")) {
  const { answer } = await import("./lsp-worker.js");
  const server = new WebAssembly.Module(readFileSync(root + "web/lz-lsp.wasm"));
  const probe = answer(server, "a = 5,\nSTOP\n", []);
  if (!probe.capabilities?.textDocumentSync) {
    lspNote = "web/lz-lsp.wasm gives no answers yet (the editor falls back to lz --check and its own colours)";
  } else {
    lspNote = "the language server answered";
    const legend = probe.capabilities.semanticTokensProvider?.legend?.tokenTypes;
    check("lsp legend", Array.isArray(legend) && legend.length > 0, JSON.stringify(probe.capabilities));
    for (const file of readdirSync(root + "examples").filter((f) => f.endsWith(".lzr")).sort()) {
      const text = readFileSync(`${root}examples/${file}`, "utf8");
      const reply = answer(server, text, [{ method: "textDocument/semanticTokens/full" }]);
      check(`lsp ${file}`, reply.results[0]?.data?.length > 0);
      // An example that does not translate on purpose has its errors in its .out file.
      const out = `${root}examples/${file.slice(0, -4)}.out`;
      const meant = settingsOf(text).rules || (existsSync(out) && /^\d+:\d+: error/.test(readFileSync(out, "utf8")));
      const errors = reply.diagnostics.filter((d) => d.severity === 1);
      check(`lsp ${file} errors`, meant ? errors.length > 0 : errors.length === 0, JSON.stringify(errors));
      asked++;
    }
    const hover = answer(server, "x = F^1(y),\nSTOP\n", [{ method: "textDocument/hover", params: { position: { line: 0, character: 5 } } }]);
    check("lsp hover", /square root/i.test(JSON.stringify(hover.results[0])), JSON.stringify(hover.results[0]));
  }
}

console.log(`${examples} examples, ${asked} files through the language server (${lspNote}), ${failed} failed`);
process.exit(failed ? 1 : 0);
