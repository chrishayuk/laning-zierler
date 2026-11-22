// The Laning and Zierler extension: starts the language server and adds
// commands that feed the tape in the editor to the system and show the typed
// page, or the translation.
"use strict";

const vscode = require("vscode");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;

function setting(name) {
  const value = vscode.workspace.getConfiguration("lz").get(name);
  return typeof value === "string" ? value.trim() : value;
}

async function startServer(context) {
  const custom = setting("server.path");
  // Built in: the server compiled to WebAssembly, run by the editor's own
  // Node, so nothing has to be installed.
  const builtIn = { module: context.asAbsolutePath(path.join("server", "lsp.js")), transport: TransportKind.stdio };
  const serverOptions = custom ? { command: custom, args: [] } : { run: builtIn, debug: builtIn };
  client = new LanguageClient("lz", "Laning–Zierler", serverOptions, {
    documentSelector: [{ language: "laning-zierler" }],
  });
  await client.start();
}

async function restartServer(context) {
  if (client) await client.stop().catch(() => {});
  client = undefined;
  await startServer(context);
}

/** Options for `lz` from the settings, and any extra ones. */
function options(extra) {
  const args = [];
  if (setting("zero") === "plus") args.push("--zero", "plus");
  if (setting("temporaryRules")) args.push("--temporary-rules");
  return args.concat(extra);
}

async function runTape(context, extra, title) {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "laning-zierler") {
    vscode.window.showInformationMessage("Open a Laning–Zierler tape (.lzr) to run it.");
    return;
  }
  const document = editor.document;
  let file = document.uri.fsPath;
  if (document.isUntitled || document.uri.scheme !== "file") {
    // Not on disk yet: run a copy.
    file = path.join(os.tmpdir(), "untitled.lzr");
    fs.writeFileSync(file, document.getText());
  } else if (document.isDirty) {
    await document.save();
  }

  const args = options(extra);
  const custom = setting("interpreter.path");
  const execution = custom
    ? new vscode.ProcessExecution(custom, [...args, file])
    : new vscode.ProcessExecution(process.execPath, [context.asAbsolutePath(path.join("server", "run.js")), ...args, file], {
        env: { ELECTRON_RUN_AS_NODE: "1" },
      });
  const task = new vscode.Task({ type: "lz" }, vscode.TaskScope.Workspace, `${title} ${path.basename(file)}`, "lz", execution, []);
  task.presentationOptions = {
    reveal: vscode.TaskRevealKind.Always,
    panel: vscode.TaskPanelKind.Dedicated,
    focus: true,
    clear: true,
    // Without this the terminal first prints the whole command line.
    echo: false,
    showReuseMessage: false,
  };
  await vscode.tasks.executeTask(task);
}

async function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand("lz.run", () => runTape(context, [], "Run")),
    vscode.commands.registerCommand("lz.trace", () => runTape(context, ["--trace", "--stats"], "Trace")),
    vscode.commands.registerCommand("lz.translate", () => runTape(context, ["--translate"], "Translate")),
    vscode.commands.registerCommand("lz.restartServer", () => restartServer(context)),
    // The server's formatter does one thing: it writes the Flexowriter's
    // characters (² for ^2, · for *) where a stand-in was typed.
    vscode.commands.registerCommand("lz.flexowriterCharacters", () => vscode.commands.executeCommand("editor.action.formatDocument")),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("lz.server.path")) restartServer(context);
    })
  );
  await startServer(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
