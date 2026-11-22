// Talks to the language server the way an editor does: over standard input
// and output. It tests the WebAssembly build the extension ships and, when
// it has been built, the native program other editors would use.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import test from "node:test";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL("..", import.meta.url));
const native = fileURLToPath(new URL("../../../target/release/lz-lsp", import.meta.url));
const servers = [["WebAssembly", process.execPath, [`${here}server/lsp.js`, "--stdio"]]];
if (existsSync(native)) servers.push(["native", native, []]);

function connect(command, args) {
  const child = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
  let buffer = Buffer.alloc(0);
  const waiting = [];
  const arrived = [];
  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const header = buffer.indexOf("\r\n\r\n");
      if (header === -1) return;
      const length = Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())[1]);
      if (buffer.length < header + 4 + length) return;
      const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString());
      buffer = buffer.subarray(header + 4 + length);
      const next = waiting.shift();
      if (next) next(message);
      else arrived.push(message);
    }
  });
  const send = (message) => {
    const body = JSON.stringify({ jsonrpc: "2.0", ...message });
    child.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  };
  const receive = () => new Promise((resolve) => (arrived.length ? resolve(arrived.shift()) : waiting.push(resolve)));
  let id = 0;
  const request = async (method, params) => {
    send({ id: ++id, method, params });
    const reply = await receive();
    assert.equal(reply.id, id);
    return reply.result;
  };
  const exited = new Promise((resolve) => child.on("exit", resolve));
  return { send, receive, request, exited, kill: () => child.kill() };
}

const uri = "file:///cosine.lzr";
// E-364 p. 11, the cosine series.
const program = `x = 0,
1   z = 1 - x^2/2 + x^4/2·3·4 - x^6/2·3·4·5·6,
    PRINT x, z.
    x = x + .1,
    e = x - 1.05,
    CP 1,
    STOP
`;

for (const [name, command, args] of servers) {
  test(`${name} server: a session from initialize to exit`, async (t) => {
    const server = connect(command, args);
    // A failed assertion must not leave the server running, or the test never ends.
    t.after(() => server.kill());
    const initialised = await server.request("initialize", { processId: null, rootUri: null, capabilities: {} });
    assert.equal(initialised.serverInfo.name, "lz-lsp");
    assert.equal(initialised.capabilities.hoverProvider, true);
    assert.equal(initialised.capabilities.inlayHintProvider, true);
    server.send({ method: "initialized", params: {} });

    server.send({ method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "laning-zierler", version: 1, text: program } } });
    const clean = await server.receive();
    assert.equal(clean.method, "textDocument/publishDiagnostics");
    assert.deepEqual(clean.params.diagnostics.filter((d) => d.severity === 1), []);

    const at = (line, character) => ({ textDocument: { uri }, position: { line, character } });
    const cp = await server.request("textDocument/hover", at(5, 4));
    assert.match(cp.contents.value, /`CP`.*negative/);
    const slash = await server.request("textDocument/hover", at(1, 15));
    assert.match(slash.contents.value, /ac divided by b/);
    // after "·" (one UTF-16 unit, two bytes) the columns still line up
    const three = await server.request("textDocument/hover", at(1, 26));
    assert.match(three.contents.value, /`3`\*\* a number/);
    const definition = await server.request("textDocument/definition", at(5, 7));
    assert.equal(definition.range.start.line, 1);
    const hints = await server.request("textDocument/inlayHint", { textDocument: { uri }, range: { start: { line: 0, character: 0 }, end: { line: 7, character: 0 } } });
    assert.deepEqual(hints.map((h) => h.label), ["101", "102", "103", "104", "105"]);
    const formatted = await server.request("textDocument/formatting", { textDocument: { uri }, options: { tabSize: 4, insertSpaces: true } });
    assert.deepEqual(formatted.map((e) => e.newText), ["²", "⁴", "⁶"]);

    server.send({ method: "textDocument/didChange", params: { textDocument: { uri, version: 2 }, contentChanges: [{ text: "x = 1\nSTOP\n" }] } });
    const broken = await server.receive();
    assert.equal(broken.params.diagnostics[0].severity, 1);

    assert.equal(await server.request("shutdown", null), null);
    server.send({ method: "exit" });
    assert.equal(await server.exited, 0);
  });
}
