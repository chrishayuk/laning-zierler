# laning-zierler

The algebraic system that J. H. Laning Jr. and N. Zierler wrote for MIT's
Whirlwind I in 1952-53, rebuilt in Rust with no dependencies from the
programmer's manual they published in January 1954, Engineering Memorandum
E-364, *A Program for Translation of Mathematical Equations for Whirlwind I*.

```
a = 5,
y = -6.3a,
b = 0.0053 (a-y) /2ay,
PRINT a, y, b.
STOP
```
```
a = +.50000000+01       y = -.31500001+02       b = -.61412696-03
```

You typed equations on a Flexowriter, which punched them on paper tape; the
tape was read into Whirlwind, translated into Whirlwind's own instructions, and
run, and the answers came back typewritten. "The effect of our program is to
create a computer within a computer." It was working by May 1953, four years
before the first FORTRAN compiler was delivered, and in 1954 John Backus,
Harlan Herrick and Irving Ziller came to MIT to see it run.

The manual ends with a section called "Temporary Rules and Restrictions": two
features not yet working, and "All other portions of the program are believed
to be operating satisfactorily but exhaustive tests have not been made.
Information relative to any troubles encountered in the use of this program
would be appreciated by the authors." This repository is, among other things,
those tests.

## Install

You need [Rust](https://rustup.rs). To install the interpreter and the language
server from this repository:

```bash
cargo install --git https://github.com/chrishayuk/laning-zierler laning-zierler lz-lsp
```

That puts `lz` and `lz-lsp` in `~/.cargo/bin`. Then:

```bash
echo 'a = 2, b = a^10, PRINT b. STOP' | lz
```

For Visual Studio Code, download `laning-zierler-language-0.1.0.vsix` from the
[latest release](https://github.com/chrishayuk/laning-zierler/releases/latest)
and install it:

```bash
code --install-extension laning-zierler-language-0.1.0.vsix
```

Open any `.lzr` file and you get highlighting, problems as you type, hover help
(hover on `=` to see the block an equation is translated into) and a run
button. The extension carries its own copy of the interpreter and language
server, so it works without the `cargo install` above. For Neovim and Helix see
[`editors/README.md`](editors/README.md).

Or try it with nothing installed, in the
[playground](https://chrishayuk.com/demos/laning-zierler).

## Build and run

```bash
cargo build --release
target/release/lz examples/1954-differential-equations.lzr
```

A `.lzr` file is a tape: what would have been punched. `lz tape` prints the
page the delayed printer typed. Other ways to run it:

| | |
|---|---|
| `lz data.lzr program.lzr` | read tapes in turn, as a data tape and then a program tape were read after a stop code |
| `lz --translate tape` | each equation's number and the block of orders it was translated into; nothing is run |
| `lz --check tape` | read the tape and say what is wrong with it, and what the manual warns about |
| `lz --trace tape` | each equation as it is carried out, with its value (to standard error) |
| `lz --stats tape` | what the run cost: equations, blocks brought from the drum, floating-point operations, drum accesses |
| `lz --zero plus tape` | every zero is +0 (the default is one's complement; see below) |
| `lz --temporary-rules tape` | the restrictions of January 1954: no `x|N`, and no `SR` or `CR` to an equation with a function |

## The language

Every lower-case letter is a variable. An equation is a letter, an equals sign,
an expression and a **comma**. Equations are carried out in the order in which
they are written. A number in front of an equation, from 1 to 100, is its
equation number, for jumps.

| | |
|---|---|
| `x = expression,` | compute and store |
| `3 x = 2,` | equation number 3 |
| `PRINT a, b, x|3.` | type the current values; ends with a **period**. No computation: `PRINT 3a.` is not allowed |
| `SP 5,` | go on from equation 5 |
| `CP 5,` | go to 5 if the last quantity computed was negative |
| `SR 5,` | carry out equation 5, then come back |
| `CR 5,` | `SR 5` if the last quantity computed was negative |
| `SP n,` | the same with the value of a letter as the number |
| `x|5`  `x|n` | a "superscript variable", x with index 5 or with the value of n (0 to 1023) |
| `ASSIGN a|1000,` | put a|1 to a|1000 on the drum |
| `g|N = 2, 4, 6, 0,` | g|1 to g|4 on the drum, with these values |
| `a|N = 1 (.5) 2 (1) 4,` | 1, 1.5, 2, 3, 4: a value, (increment), up to |
| `D y = expression,` | dy/dt: a differential equation (`t` is the independent variable, `h` the step) |
| `F^2(x)` | function subroutine 2, the sine |
| `STOP` | the end, as the last thing on the tape |

Arithmetic is `+`, `-`, `/` and multiplication by putting things side by side:
`2ay` is 2 · a · y, and `0.5 (x + 2/x)` is what it looks like. `·` and `:`
also mean times. No more than four parentheses may be open at once.

**A slash is not what you think.** Plus and minus separate terms; inside a term,
a slash moves everything after it to the other side of the fraction, until the
next slash. So `cd/a(b + c)` is cd over a(b + c), and

> "a/b/c is properly interpreted as the product ac divided by b" (p. 4-5)

which FORTRAN would make a/(bc), and so would most people. See
`examples/1954-division.lzr`.

**A space can be a multiplication sign.** Spaces, tabs and carriage returns are
ignored, "except when they occur between the digits of a number": `x = 23,` is
twenty-three and `x = 2 3,` is six.

### Typing the Flexowriter's characters

The Flexowriter had upper-case digits, which typed as superscripts and were
exponents, and a vertical bar. A keyboard has neither superscripts nor (on the
same key) the bar's meaning, so:

| Flexowriter | meaning | type |
|---|---|---|
| `x²`, `x⁻³` | exponents (integers only) | `x^2`, `x^-3`, or the superscript characters |
| `F²(x)` | function subroutine 2 | `F^2(x)` |
| `x|⁵` | superscript variable 5 | `x|5` |
| `·` | times (a colon with its lower dot filed off) | `·`, `:` or `*` |
| nullify | ignored, even inside a number | DEL (0x7F) |

A `#` starts a note that runs to the end of the line. Notes are not part of the
tape; there was no such thing.

### The functions

| | | | | | |
|---|---|---|---|---|---|
| F¹ square root | F² sine | F³ cosine | F⁴ tangent | F⁵ sin⁻¹ | F⁶ cos⁻¹ |
| F⁷ tan⁻¹ | F⁸ secant | F⁹ cosecant | F¹⁰ cotangent | F¹¹ absolute value | F¹² signum |
| F¹³ eˣ | F¹⁴ e⁻ˣ | F¹⁵ sinh | F¹⁶ cosh | F¹⁷ tanh | F¹⁸ sech |
| F¹⁹ csch | F²⁰ coth | F²¹ log₁₀ | F²² log₂ | F²³ logₑ | |

### Numbers

"The 24, 6 number system": a 24-bit mantissa and a binary exponent from -63 to
63, so about seven decimal digits, and nothing beyond 2⁶³ - 2³⁹ (about
9.22 · 10¹⁸) in size. Printed, a number is a sign, eight digits and a power of
ten: `+.50000000+01` is 5. Adding .1 ten times does not give 1:

```
x = +.90000009+00       z = +.62160986+00

x = +.10000001+01       z = +.54030215+00
```

A number is read "as an integer, placing the decimal point only after the
number is completely read in", so a number with more than nineteen digits is
too large even with a point in it.

### Differential equations

"Provision has been made for the solution of ordinary differential equations
by the method of Gill - a variation of the 4th order Runge-Kutta technique."
Write each first-order equation with `D`. The stretch of program from the first
`D` equation to the last is gone through four times for each step of `h`, and
`t` goes up by `h` at the end of it. Anything the derivatives need must be
computed between the first and last `D` equations: the manual gives a change
that "would appear to be innocuous" and gets the wrong answer, and so does
`examples/1954-innocuous-change.lzr`. `lz --check` warns about it.

### Zero

Whirlwind was a one's-complement machine, with a +0 and a -0, and the manual
says so twice. `CP` jumps on a negative number, and on a zero "the CP may or
may not behave like an SP since 0 may appear as +0 or -0" (p. 8). And a zero
index is permitted "if assigned by the specific equation n = 0," and not
otherwise (p. 13).

Here a zero has a sign. A constant 0 is +0; the difference of two equal
numbers, as a one's-complement adder makes it, is -0. So `e = n - 5,` with `n`
equal to 5 gives -0, `CP` jumps, and `PRINT e.` types `-.00000000+00`. The
manual's own loops step round this: they test `n - 9.5`, never `n - 10`.
`lz --zero plus` makes every zero +0 instead.

### Post-mortems

An arithmetic alarm (a number too large, a division by zero, the square root of
a negative number) stops the program, and the programmer "will be told the
number of the equation in which the alarm occurred and the number of the
equation that preceded the alarm". Equations without numbers get them, 101 to
200, "in the order in which they are written". If equation 100 is a `PRINT`,
it is carried out after an alarm.

```
ALARM  EQUATION 1  PRECEDED BY 103

n = +.62000000+02       x = +.46116860+19
```

## What the tests found

The manual's examples run. The table of cosines, the iterative cosine, the
interpolation by a rounded index, the drum, the two differential equations of
section 15 (whose answers are sin(t - 2) and cos(t - 2) - 1; at t = 3 the
reconstruction gives .84103733 and -.45941162, against .84147098 and
-.45969769, which is what a step of 0.5 should cost) and the innocuous change
that breaks them. Some things are less tidy.

**Knuth's TPK stops on its last number.** In 1976 Knuth and Trabb Pardo wrote
their TPK algorithm in the Laning and Zierler system, to show what it looked
like (`examples/1976-knuth-tpk.lzr`). It reads eleven numbers with `v|N` and
runs back down them with `i = i - 1` and `a|i`. The last time round, `i` is a
zero computed by arithmetic, which the manual does not permit as an index: the
program types ten lines and then an alarm. Under the rules of January 1954 it
does not translate at all, because `v|N` "is not yet in operation", and the
first-quarter report of 1954 says the same thing of the x|N routine. TPK was
written as an illustration twenty-two years later and was never run on
Whirlwind; this says only what the manual's rules make of it. With
`--zero plus` it runs to the end, but that is a what-if, not the manual: the
page-13 rule forbids a computed zero index whatever its sign, and `--zero plus`
lets it through because the zero it computes is +0, the sign a zero set by
`n = 0,` has.

**The manual's figure for the largest number is wrong.** Page 6 gives
2⁶³ - 2³⁹ as "(or, roughly, 9.2233995 · 10¹⁸)". It is 9223371487098961920, or
9.2233715 · 10¹⁸. The integer the page gives next as "a permissible number",
9223314040000000000, is below both, so nothing else on the page depends on it.
(2⁶³ itself is 9.2233720 · 10¹⁸, so it is not that either.)

**The manual's exponent examples, run in order, stop with an alarm.** Section 2D gives
`a = 5²,` `b = (a - 2)⁻²,` `c = (ab³)⁻⁵,`. (ab³)⁵ is about 10⁻³⁴, which the 24,
6 system cannot hold; it becomes zero and `c` divides by it. They are notation,
not a program, but they are a reminder of how small the range was.

**The summer-school copy of the D-equation program types nothing.** The MIT
Summer Session notes of 1954 reprint the program of section 15 without its
`PRINT`, and without the commas the rules require on its last two lines.

**Printing truncates.** The manual prints .05 as `+.50000000-01`. The 24-bit
number nearest .05 is .0500000007..., so a conversion that rounded would type
`+.50000001-01`. Truncating to eight digits gives both values the manual prints
(.05 and -π), and that is what this does.

## What is not original

E-364 is a manual, not a listing. The translator's code has not been found:
"there are no annotated copies of the conversion routine" (Summer Session
1954), and Zierler's report on its "inner workings" was started and, as far as
is known, not finished. So everything below is this reconstruction's choice.

- **The orders.** Equations are translated into blocks of a small stack code,
  one order per floating-point routine called, and the blocks are what runs.
  That much is the shape Knuth and Trabb Pardo describe ("blocks of subroutine
  calls, stored on the drum ... one equation's worth at a time"); the orders
  themselves are not Whirlwind's. `lz --translate` shows them.
- **Rounding.** Results are rounded to 24 bits, half away from zero. The
  interpretive routines may have truncated.
- **The sign of zero.** The one's-complement rule above is an inference from
  the machine and the manual's two warnings, not from a listing. A difference
  of equal numbers, and any sum with a -0 in it, is -0; products and quotients
  take the sign rule of the operands, and |x| is +0. The index rule is enforced
  through the sign: a zero index is allowed when it is +0, which `n = 0,` gives
  and `n = n - 1,` does not. So the TPK alarm rests on this inference as well
  as on page 13. Under `--zero plus` every zero is +0,
  so that mode also allows zero indices the manual forbids.
- **Gill's method.** The formulas are Gill's (1951) in the usual form, with the
  `q` that carries rounding error from one step to the next. Each `D` equation
  stores h·f; at the last `D` equation of a pass all the dependent variables
  move together. `t` is t + h/2 for the second and third passes and t + h for
  the fourth. The manual says only that the stretch is gone through four times
  and that t goes up by h at the end; the two storage locations per `D`
  equation ("250 - 2k") fit this.
- **The function subroutines** are computed in double precision from the 24-bit
  argument and rounded. The 1954 routines are not known.
- **Underflow** becomes a zero of the same sign, silently, so halving a number
  forever never alarms. E-364 says nothing about numbers too small.
- **An index of exactly n + .5** rounds up: `k|j` with j = 2.5 is k|3. The
  manual says only that the computer "rounds n off to the nearest integer".
- **t is added to, not recomputed.** Each step sets t to t + h in 24 bits, so
  over a long run t drifts: 160 steps of .05 end at `+.80000123+01`.
- **Unset variables** read as zero. The manual says a letter must be given a
  value before it is used; `lz --check` warns when it is not.
- **The post-mortem's wording** (`ALARM  EQUATION 1  PRECEDED BY 103`) is made
  up; the manual says what it reports, not how. The reason, in words, goes to
  standard error. Which statements get the automatic numbers 101 to 200 is a
  choice too: here equations, `PRINT` and `STOP` do, and jumps, `ASSIGN` and
  `x|N` do not.
- **Storage.** The 250 superscript variables in core, less two for each `D`
  equation and one for each group on the drum, are the manual's. How much drum
  an equation took is not known: here it is one variable's worth for every two
  orders, against the drum's 3072.
- **The layout of a PRINT line**: six values to a line in columns 24
  characters apart. The manual shows six to a line and says nothing of the
  spacing.
- **The stop code** (Flexowriter character 49, which halted the reader so that
  a program tape could follow a data tape) is the command line's list of tapes.
- **Messages** for a tape that cannot be translated are this reconstruction's.
  What the translator of 1954 did with a mistyped tape is not recorded.
- **Time** is not simulated. `lz --stats` counts the work; the only time in the
  manual is that a variable on the drum costs "about 30 milliseconds longer"
  per use, and `--stats` multiplies that out.

## Examples

[`examples/`](examples/) has every program in E-364 that can be run, under
`1954-`, Knuth and Trabb Pardo's TPK under `1976-`, and programs written for
this repository. Each has a `.out` file with its page, and `cargo test` runs
them all.

## Tutorial

[`tutorial/`](tutorial/01-introduction.md) goes from `a = 5,` to differential
equations and post-mortems. Every program in it is followed by its page, and
`cargo test` runs them.

## In the browser

`web/` runs the system in a web page, as a batch job, which is what it was.
You type the tape on the left, a Flexowriter with colours, problems, hover help
and completion from the language server running in the page. Feed it: the
punched tape passes the reader, the tape is translated into blocks on the drum,
and the delayed printer types the answers at ten characters a second (or fast,
or all at once). An alarm is typed as the post-mortem, the two equation
numbers, with the reason in a margin note. The tabs show the translation (each
equation's number and its block of orders), a trace, what the run cost, and the
same tape fed with each zero model side by side. That last one is where Knuth's
TPK parts company with itself. `lz` is compiled unchanged for WASI, and
`web/wasi.js` is a small host that supplies standard input and output. The
holes in the paper tape are illustrative, not the Flexowriter's code.

```bash
node web/build.mjs                  # builds web/lz.wasm, web/lz-lsp.wasm, web/examples.json
node web/test.mjs                   # feeds every example through the wasm build and the language server
python3 -m http.server -d web 8000  # then open http://localhost:8000
```

Add `?example=1976-knuth-tpk&run` to the address to feed a tape at once, and
`&speed=fast`, `&zero=plus` or `&rules=1954`.

## In your editor

`lz --check` drives a language server, `lz-lsp` in [`lsp/`](lsp/), and a Visual
Studio Code extension in [`editors/vscode`](editors/vscode). See
[`editors/README.md`](editors/README.md).

The interpreter itself has no dependencies. The language server is a separate
crate in the same workspace and uses `serde_json`.

## Sources

[`sources/README.md`](sources/README.md) lists the four documents, where to get
them (`sources/fetch.sh`), and what each adds. [`docs/scans/`](docs/scans/README.md)
has the pages that matter most, reduced.

The pasted research that started this said the lead-pursuit program was 97
statements and that the system was ten times slower than hand code. Knuth and
Trabb Pardo, quoting Laning, give 79 statements with seven uses of the
differential-equation feature. The ten-to-one is Charles Adams's, in a talk at
the Office of Naval Research symposium of May 1954, against "an efficient
computer program"; the paper names Laning as co-author, but Knuth and Trabb
Pardo record that he "did not write the paper or attend the conference". The
summer school of 1954 put it at six to eight times the Digital Computer
Laboratory's interpretive system.

All of the reading was done by a coding agent and its sub-agents in one
session, from the page images. Nobody has yet checked it against the pages by
eye.

## Tests

```bash
cargo test
```

The unit tests cover the 24, 6 arithmetic and its zeros, the printed form, the
reading of the tape (the slash, the spaces between digits, the four
parentheses, lists), translation and the run. The integration tests run every
example and the tutorial and compare pages, and drive the command line.

## Licence

Apache License 2.0 - see [LICENSE](LICENSE). The page images in `docs/scans`
are reproduced for reference from the documents listed in `sources/`.
