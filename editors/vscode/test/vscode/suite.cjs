// Runs inside VS Code. Exercises the extension the way a person would: open
// a tape, hover, jump, rename, see problems and the translator's numbers,
// and run it.
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function until(what, probe, seconds = 20) {
  for (let i = 0; i < seconds * 10; i++) {
    const value = await probe();
    if (value) return value;
    await sleep(100);
  }
  throw new Error(`timed out waiting for ${what}`);
}

const hoverText = (hovers) =>
  hovers.flatMap((h) => h.contents).map((c) => (typeof c === "string" ? c : c.value)).join("\n");

exports.run = async function run() {
  const folder = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const document = await vscode.workspace.openTextDocument(path.join(folder, "1954-differential-equations.lzr"));
  await vscode.window.showTextDocument(document);
  assert.equal(document.languageId, "laning-zierler", "the .lzr extension selects the language");

  const line = (text) => {
    for (let i = 0; i < document.lineCount; i++) if (document.lineAt(i).text.startsWith(text)) return i;
    throw new Error(`no line ${text}`);
  };
  const cp3 = line("    CP 3,");
  const stop = line("3   STOP");
  const target = new vscode.Position(cp3, 7); // the 3 of "CP 3,"

  // The server is ready when it answers a hover.
  const hovers = await until("a hover from the language server", async () => {
    const found = await vscode.commands.executeCommand("vscode.executeHoverProvider", document.uri, target);
    return found.length ? found : undefined;
  });
  assert.match(hoverText(hovers), /Equation 3/);
  const d = await vscode.commands.executeCommand("vscode.executeHoverProvider", document.uri, new vscode.Position(line("2   D y|1"), 4));
  assert.match(hoverText(d), /Gill/);
  console.log("hover ok");

  const definitions = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", document.uri, target);
  assert.equal((definitions[0].range || definitions[0].targetRange).start.line, stop);
  const references = await vscode.commands.executeCommand("vscode.executeReferenceProvider", document.uri, target);
  assert.equal(references.length, 2, "the number in front of STOP, and CP 3");
  const edit = await vscode.commands.executeCommand("vscode.executeDocumentRenameProvider", document.uri, target, "9");
  assert.equal(edit.entries()[0][1].length, 2);
  console.log("definition, references and rename ok");

  const hints = await vscode.commands.executeCommand("vscode.executeInlayHintProvider", document.uri, new vscode.Range(0, 0, document.lineCount, 0));
  const labels = hints.map((h) => (typeof h.label === "string" ? h.label : h.label.map((p) => p.value).join("")));
  assert.equal(labels[0], "101", "the translator numbers the first unnumbered equation 101");
  console.log("inlay hints ok");

  const legend = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokensLegend", document.uri);
  assert.ok(legend.tokenTypes.includes("label"));
  const symbols = await vscode.commands.executeCommand("vscode.executeDocumentSymbolProvider", document.uri);
  assert.ok(symbols.map((s) => s.name).includes("3  STOP"));
  console.log("symbols and semantic tokens ok");

  // A tape with a mistake shows it.
  const broken = await vscode.workspace.openTextDocument({ language: "laning-zierler", content: "x = a^2 * b\nSTOP\n" });
  await vscode.window.showTextDocument(broken);
  const problems = await until("diagnostics", () => {
    const found = vscode.languages.getDiagnostics(broken.uri).filter((p) => p.severity === vscode.DiagnosticSeverity.Error);
    return found.length ? found : undefined;
  });
  assert.equal(problems[0].source, "lz");
  console.log("diagnostics ok");

  // The formatter writes the Flexowriter's characters for the stand-ins.
  const edits = await vscode.commands.executeCommand("vscode.executeFormatDocumentProvider", broken.uri, { tabSize: 4, insertSpaces: true });
  assert.deepEqual(edits.map((e) => e.newText), ["²", "·"]);
  console.log("formatting ok");

  // Run Tape: a task that ends with exit code 0.
  await vscode.window.showTextDocument(document);
  const finished = new Promise((resolve) => {
    const listener = vscode.tasks.onDidEndTaskProcess((event) => {
      if (event.execution.task.definition.type === "lz") {
        listener.dispose();
        resolve(event.exitCode);
      }
    });
  });
  await vscode.commands.executeCommand("lz.run");
  assert.equal(await Promise.race([finished, sleep(20000).then(() => "timed out")]), 0);
  console.log("run ok");
};
