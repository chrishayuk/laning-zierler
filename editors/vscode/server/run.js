// Feeds a tape to the Laning and Zierler system:  node run.js [options] tape.lzr
// The tape is given to the interpreter on standard input, so it needs no
// access to files, and it prints the typed page. Options are those of `lz`
// (--zero plus, --temporary-rules, --translate, --trace, --stats, --check).
"use strict";

const fs = require("fs");
const path = require("path");
const { run } = require("./wasi.js");

const args = process.argv.slice(2);
const file = args.pop();
if (!file || file.startsWith("--")) {
  console.error("usage: run.js [options] tape.lzr");
  process.exit(2);
}
let tape;
try {
  tape = fs.readFileSync(file);
} catch (error) {
  console.error(`lz: cannot read ${file}: ${error.message}`);
  process.exit(2);
}
process.exitCode = run(path.join(__dirname, "lz.wasm"), { args, prefix: tape, keyboard: false });
