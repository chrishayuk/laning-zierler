// Builds everything the extension ships:  node build.mjs
//   server/lz-lsp.wasm   the language server, compiled for WASI
//   server/lz.wasm       the interpreter, compiled for WASI
//   dist/extension.js    the extension and its one dependency, bundled
import { execSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

// fileURLToPath, not .pathname: the folder name has an en dash in it.
const here = fileURLToPath(new URL(".", import.meta.url));
const root = fileURLToPath(new URL("../..", import.meta.url));
const release = `${root}target/wasm32-wasip1/release`;

execSync("cargo build --release --target wasm32-wasip1 -p laning-zierler -p lz-lsp", { cwd: root, stdio: "inherit" });
copyFileSync(`${release}/lz-lsp.wasm`, `${here}server/lz-lsp.wasm`);
copyFileSync(`${release}/lz.wasm`, `${here}server/lz.wasm`);

await build({
  entryPoints: [`${here}src/extension.js`],
  outfile: `${here}dist/extension.js`,
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node18",
  external: ["vscode"],
  minify: true,
  logLevel: "warning",
});
console.log("server/lz-lsp.wasm, server/lz.wasm and dist/extension.js written");
