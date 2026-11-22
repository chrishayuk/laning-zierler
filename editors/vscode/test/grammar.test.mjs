// Tokenises tapes with the same TextMate engine VS Code uses and checks the
// scopes the grammar assigns.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";
import { fileURLToPath } from "node:url";
import oniguruma from "vscode-oniguruma";
import textmate from "vscode-textmate";

const require = createRequire(import.meta.url);
const grammarPath = fileURLToPath(new URL("../syntaxes/laning-zierler.tmLanguage.json", import.meta.url));
const examples = fileURLToPath(new URL("../../../examples/", import.meta.url));

const wasm = readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"));
await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
const registry = new textmate.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (text) => new oniguruma.OnigString(text),
  }),
  loadGrammar: async () => textmate.parseRawGrammar(readFileSync(grammarPath, "utf8"), grammarPath),
});
const grammar = await registry.loadGrammar("source.lzr");

/** Each non-blank piece of text with its most specific scope. */
function scopes(source) {
  const out = [];
  let state = textmate.INITIAL;
  for (const line of source.split("\n")) {
    const result = grammar.tokenizeLine(line, state);
    for (const token of result.tokens) {
      const text = line.slice(token.startIndex, token.endIndex);
      if (text.trim()) out.push([text.trim(), token.scopes.at(-1)]);
    }
    state = result.ruleStack;
  }
  return out;
}

test("an equation: its number, letters, numbers, the slash and the comma", () => {
  assert.deepEqual(scopes("5y = 0.0053(a-y)/2ay,"), [
    ["5", "entity.name.section.equation.lzr"],
    ["y", "variable.other.letter.lzr"],
    ["=", "keyword.operator.assignment.lzr"],
    ["0.0053", "constant.numeric.lzr"],
    ["(", "punctuation.section.parens.lzr"],
    ["a", "variable.other.letter.lzr"],
    ["-", "keyword.operator.arithmetic.lzr"],
    ["y", "variable.other.letter.lzr"],
    [")", "punctuation.section.parens.lzr"],
    ["/", "keyword.operator.division.lzr"],
    ["2", "constant.numeric.lzr"],
    ["a", "variable.other.letter.lzr"],
    ["y", "variable.other.letter.lzr"],
    [",", "punctuation.terminator.equation.lzr"],
  ]);
});

test("functions, exponents in both spellings, and times", () => {
  const s = scopes("x = F^1(F¹¹(a|i))+5(a|i)^3 · b⁻²,");
  assert.deepEqual(s.slice(2, 4), [["F", "support.function.lzr"], ["^1", "support.function.number.lzr"]]);
  assert.deepEqual(s.slice(5, 7), [["F", "support.function.lzr"], ["¹¹", "support.function.number.lzr"]]);
  assert.deepEqual(s.slice(8, 11), [["a", "variable.other.letter.lzr"], ["|", "punctuation.separator.superscript.lzr"], ["i", "variable.parameter.index.lzr"]]);
  assert.ok(s.some(([t, k]) => t === "^3" && k === "keyword.operator.power.lzr"));
  assert.ok(s.some(([t, k]) => t === "⁻²" && k === "keyword.operator.power.lzr"));
  assert.ok(s.some(([t, k]) => t === "·" && k === "keyword.operator.times.lzr"));
});

test("instructions, jumps, D, lists and notes", () => {
  assert.deepEqual(scopes("2   D y|1 = y|2 + 1,   # dy1/dt"), [
    ["2", "entity.name.section.equation.lzr"],
    ["D", "keyword.other.derivative.lzr"],
    ["y", "variable.other.letter.lzr"],
    ["|", "punctuation.separator.superscript.lzr"],
    ["1", "constant.numeric.superscript.lzr"],
    ["=", "keyword.operator.assignment.lzr"],
    ["y", "variable.other.letter.lzr"],
    ["|", "punctuation.separator.superscript.lzr"],
    ["2", "constant.numeric.superscript.lzr"],
    ["+", "keyword.operator.arithmetic.lzr"],
    ["1", "constant.numeric.lzr"],
    [",", "punctuation.terminator.equation.lzr"],
    ["# dy1/dt", "comment.line.number-sign.lzr"],
  ]);
  assert.deepEqual(scopes("CP 1, SR u, PRINT x. 3 STOP"), [
    ["CP", "keyword.control.jump.lzr"],
    ["1", "entity.name.section.equation.reference.lzr"],
    [",", "punctuation.terminator.equation.lzr"],
    ["SR", "keyword.control.jump.lzr"],
    ["u", "variable.other.letter.lzr"],
    [",", "punctuation.terminator.equation.lzr"],
    ["PRINT", "keyword.control.lzr"],
    ["x", "variable.other.letter.lzr"],
    [".", "punctuation.terminator.print.lzr"],
    ["3", "entity.name.section.equation.lzr"],
    ["STOP", "keyword.control.lzr"],
  ]);
  const list = scopes("a|N = 1.0 (.5) 2. (0.25) 2.5,");
  assert.deepEqual(list[2], ["N", "keyword.other.list.lzr"]);
  // values in a list are numbers, not equation numbers
  assert.ok(list.filter(([, k]) => k === "entity.name.section.equation.lzr").length === 0);
  assert.deepEqual(scopes("ASSIGN g|4,")[0], ["ASSIGN", "keyword.control.lzr"]);
  assert.deepEqual(scopes("x = Q,")[2], ["Q", "invalid.illegal.lzr"]);
});

test("every example tokenises with nothing marked invalid", () => {
  const files = readdirSync(examples).filter((f) => f.endsWith(".lzr"));
  assert.ok(files.length >= 10);
  for (const file of files) {
    const bad = scopes(readFileSync(examples + file, "utf8")).filter(([, k]) => k === "invalid.illegal.lzr");
    assert.deepEqual(bad, [], file);
  }
});
