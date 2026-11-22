# Laning–Zierler in your editor

Two pieces give an editor an understanding of the language:

- **`lz-lsp`**, a language server in [`../lsp`](../lsp). It works with any editor that speaks the Language Server Protocol.
- **A Visual Studio Code extension** in [`vscode`](vscode), which bundles the server and the interpreter and adds syntax highlighting and run buttons. Its [README](vscode/README.md) lists the features.

Both are built on the reconstruction itself: its lexer and parser read the tape, its translator numbers the equations and makes their blocks of orders, and [`src/check.rs`](../src/check.rs), the code behind `lz --check`, finds what is wrong. So the editor and `lz` cannot disagree about what is on the tape. What the words mean is in [`src/info.rs`](../src/info.rs), quoted from E-364. The translator of 1954 said only which equation an alarm was in. The editor says where, and why.

A `.lzr` file is a tape: equations ending in commas, PRINTs ending in periods, jumps, and STOP at the end. Line breaks mean nothing to the translator. `#` starts a note to the end of the line; notes are not on the tape.

## The language server

```sh
cargo install --git https://github.com/chrishayuk/laning-zierler lz-lsp
```

or, in a clone of this repository, `cargo build --release -p lz-lsp`.

It talks on standard input and output and takes no arguments. It provides diagnostics, hover, go to definition, references, document highlights, rename, completion, signature help, document symbols, folding ranges, inlay hints, semantic tokens, formatting and code actions. Semantic tokens mean an editor with no grammar for the language still gets highlighting; inlay hints show the numbers 101 to 200 that the translator gives the equations the programmer did not number.

Formatting does one thing. It writes the characters of the Flexowriter where a stand-in was typed: `²` for `^2`, `⁻³` for `^-3`, `F²` for `F^2`, `·` for `*`. Notes are left alone.

It also builds for WASI (`cargo build --release --target wasm32-wasip1 -p lz-lsp`), which is how the VS Code extension and the web playground run it.

### Neovim (0.11 or later)

```lua
vim.filetype.add({ extension = { lzr = "lzr" } })
vim.lsp.config("lz", {
  cmd = { "/path/to/lz-lsp" },
  filetypes = { "lzr" },
})
vim.lsp.enable("lz")
-- the translator's numbers for unnumbered equations
vim.lsp.inlay_hint.enable(true)
```

### Helix

In `languages.toml`:

```toml
[language-server.lz-lsp]
command = "/path/to/lz-lsp"

[[language]]
name = "laning-zierler"
scope = "source.lzr"
file-types = ["lzr"]
comment-token = "#"
language-servers = ["lz-lsp"]
```

### Any other LSP client

Start `lz-lsp` with no arguments for files that end in `.lzr`, with the language id `laning-zierler`. The server wants the whole text on every change (full document sync) and counts positions in UTF-16 code units.

These configurations follow each editor's documented format but have not been run here; the server itself is tested over standard input and output by `cargo test -p lz-lsp` and `vscode/test/lsp.test.mjs`.

## Typing the Flexowriter's characters

The Flexowriter typed exponents in upper case: `x²`, `a⁻³`, `F²(x)`, `x|⁵`. A keyboard today does not, so `^2`, `^-3`, `F^2(x)` and `x|5` are accepted for them, and `*` for `·`. The formatter writes the real characters for the exponents and the times; `lz` accepts both.

## Checking a tape from the command line

```sh
lz --check tape.lzr
```

prints one line per problem, as `file:line:column: severity: message`, and exits with status 1 if any is an error.
