# Laning–Zierler for Visual Studio Code

Editor support for the algebraic system of J. H. Laning Jr. and N. Zierler: the language of equations that MIT's Whirlwind I translated and ran from 1953, as its manual, Engineering Memorandum E-364 (January 1954), describes it. A `.lzr` file is a tape: what would be punched on a Flexowriter and fed to the reader.

Nothing else needs installing: the language server and the interpreter are built into the extension as WebAssembly.

## What you get

- **Syntax highlighting** for equation numbers, letters, superscript variables (`x|5`, `x|n`, `x|N`), numbers, exponents in either spelling (`x²` or `x^2`), the function subroutines `F¹` to `F²³`, `D`, the instructions, and `#` notes.
- **Hover** on anything to see what it does, in the words of E-364. On a jump's number, the equation it goes to. On a letter, where it is given a value (and, with D equations, what `t` and `h` mean). On a number, the value the 24, 6 system holds for it: `0.1` is `+.10000000+00`, exactly 0.10000000149011612. On the `=` of an equation, the block of interpretive orders the translator makes of it.
- **The translator's numbers.** Equations you do not number are numbered 101 to 200 in the order written, and an alarm names them by those numbers. They are shown in front of each unnumbered equation.
- **Problems as you type:** what stops the tape translating (a missing comma, a fifth open parenthesis, an `SP` to an equation that is not there, no `STOP`), and what the manual warns about: a letter used before it is given a value, auxiliary computation outside the D equations (the change that "would appear to be innocuous", p. 19), and a `CP` after a difference that may be exactly zero, with a quick fix that subtracts a half as the manual's own loops do.
- **Go to definition**, **find all references** and **rename** for equation numbers and letters.
- **Completion:** instructions and templates at the start of an equation, equation numbers after `SP`, `CP`, `SR` and `CR`, the 23 functions with their names, and letters. **Signature help** inside `F²( )`.
- **The Flexowriter's characters.** **Format Document** writes `²` for `^2`, `⁻³` for `^-3` and `·` for `*`. Alt+8 types `·`. Snippets `sq`, `cube`, `inv` and `times`.
- **Run Tape:** the play button at the top of the editor, or Cmd+Enter (Ctrl+Enter on Windows and Linux). The page appears in a terminal as the delayed printer would have typed it. **Run Tape with a Trace** shows each equation as it is carried out and what the run cost; **Show the Translation** lists each equation's block of orders.

## Settings

| Setting | What it does |
| --- | --- |
| `lz.zero` | `ones` (the default): a zero computed by arithmetic is -0, as on a one's-complement machine, so `CP` jumps on it. `plus`: every zero is +0. |
| `lz.temporaryRules` | The restrictions of January 1954: no `x|N`, and no `SR` or `CR` to an equation with a function. |
| `lz.server.path` | Use an `lz-lsp` program you built yourself as the language server. |
| `lz.interpreter.path` | Run tapes with an `lz` program you built yourself. |
| `lz.trace.server` | Log the messages exchanged with the language server. |

## Building it

From this folder, with Rust (and its `wasm32-wasip1` target) and Node installed:

```sh
npm install
npm run build        # compiles the server and interpreter to WebAssembly, bundles the extension
npm test             # grammar, language server and runner tests
npm run test:vscode  # opens a real VS Code and drives the extension
npm run package      # writes laning-zierler-language-0.1.0.vsix
code --install-extension laning-zierler-language-0.1.0.vsix
```

To try it without installing, open this folder in VS Code and press F5.

## How it fits together

| File | What it is |
| --- | --- |
| `syntaxes/laning-zierler.tmLanguage.json` | The highlighting grammar. |
| `language-configuration.json` | The note character and parentheses. |
| `snippets/laning-zierler.code-snippets` | `·`, `²`, `³`, `⁻¹`, a counted loop and D equations. |
| `src/extension.js` | Starts the language server and provides the run commands. |
| `server/lsp.js`, `server/lz-lsp.wasm` | The language server from `../../lsp`, compiled for WASI. |
| `server/run.js`, `server/lz.wasm` | The interpreter, compiled for WASI. |
| `server/wasi.js` | A small WASI host for Node, with no dependencies. |

## Licence

Apache License 2.0. See [LICENSE](LICENSE).
