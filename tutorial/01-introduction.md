# 1. Introduction

```lzr
a = 5,
y = -6.3a,
b = 0.0053 (a-y) /2ay,
PRINT a, y, b.
STOP
```
```page

a = +.50000000+01       y = -.31500001+02       b = -.61412696-03
```

That is a program for MIT's Whirlwind I, and the page it typed. The first
three lines are the first example in the manual (p. 2). The equations are done
in the order they are written, `PRINT` types the values, and `STOP` ends the
program. `y` is -31.500001, not -31.5, and the reason is [chapter 4](04-numbers.md).

## What this was

In 1952 J. Halcombe Laning Jr., of the MIT Instrumentation Laboratory, wrote a
program for Whirlwind that read mathematical equations, typed in ordinary
notation, and did what they said. By May 1953 he and Neal Zierler had made it
into a system other people could use (KTP 76, p. 58), and in January 1954 they
described it in Engineering Memorandum E-364, *A Program for Translation of
Mathematical Equations for Whirlwind I*:

> Nearly everyone who has had a problem to solve on a large-scale digital
> computing machine has probably felt that it would be indeed convenient if
> one could give the machine his problem in ordinary mathematical language
> with perhaps a suggestion for a method of solution. ... The effect of our
> program is to create a computer within a computer and the purpose of the
> present report is to describe this computer and provide a programmer's
> manual for its use. (p. 1)

This tutorial is about that computer within a computer. Everything in it runs
on the reconstruction in this repository, `lz`.

## How a problem was run

You typed your problem on a Flexowriter, "a specially designed typewriter that
produces a coded pattern of holes in the tape for each character (including
backspace, carriage return, etc.) that is typed" (p. 1). The paper tape went to
Whirlwind with a request. By the first quarter of 1954 that meant: load the
master tape 108-60-30, put your tape in the photoelectric reader, press start
(SR 37, p. 17). The system read the whole tape, translated every equation and
stored it on the magnetic drum, and then carried the equations out. Its
answers went to magnetic tape, and the "delayed printer" typed them afterwards
(p. 6).

So there is no conversation. You hand over a tape and get back a page.

## The sources

The reconstruction was written from four documents. Page numbers in this
tutorial are the documents' own, and a page number alone is E-364's.

| | |
|---|---|
| E-364 | J. H. Laning Jr. and N. Zierler, *A Program for Translation of Mathematical Equations for Whirlwind I*, Engineering Memorandum E-364, MIT Instrumentation Laboratory, January 1954 |
| SR 37 | Project Whirlwind Summary Report No. 37, first quarter 1954, item 108, "An Interpretive Program" |
| SS 54 | *Digital Computers: Advanced Coding Techniques*, MIT Summer Session 1954, lecture 13, "The Algebraic Coding System of Laning and Zierler", and the discussion after it |
| KTP 76 | D. E. Knuth and L. Trabb Pardo, *The Early Development of Programming Languages*, Stanford STAN-CS-76-562, August 1976, pp. 55-61 |

Where to get them, and what each adds, is in
[`sources/README.md`](../sources/README.md).

## Running `lz`

Build it once, from the top of the repository:

```bash
cargo build --release
```

`lz` takes a tape (a text file) and types the page:

```bash
target/release/lz examples/1954-first-equations.lzr
```

With no file it reads the tape from standard input. A `#` starts a note that
runs to the end of the line. Notes are not part of the tape; there was no such
thing. They are for the people reading the file.

| | |
|---|---|
| `lz tape` | translate and run; type the page |
| `lz data program` | several tapes, read one after another, as a data tape and then a program tape were ([chapter 10](10-the-drum.md)) |
| `lz --translate tape` | show each equation's number and the block of orders it was translated into |
| `lz --check tape` | report problems, warnings and hints without running |
| `lz --trace tape` | show each equation as it is carried out |
| `lz --stats tape` | count what the run cost |
| `lz --zero plus tape` | make every zero +0 ([chapter 8](08-zero.md)) |
| `lz --temporary-rules tape` | the restrictions of January 1954 ([chapter 13](13-temporary-rules.md)) |
| `lz --limit N tape` | give up after N equations (5,000,000 unless told) |

## How to read this tutorial

Every sitting is shown as two blocks. The first is the tape: you can save it as
a file and run it. The second, straight after it, is everything `lz` writes for
it, in the order it writes it.

```lzr
x = 2 3,
PRINT x.
STOP
```
```page

x = +.60000000+01
```

The page starts with an empty line because `PRINT` begins with a carriage
return ([chapter 6](06-print.md)). The lines after it are what the delayed
printer typed.

Some lines on a page are not the printer's. `lz` adds them:

- `<stdin>:2:7: error: ...` is a tape that cannot be translated, at line 2,
  column 7. Nothing is run. `warning:` and `hint:` lines come from
  `lz --check`: a warning is something the manual says is wrong, a hint is
  something it says to be careful of. They do not stop a tape.
- `ALARM  EQUATION 103  PRECEDED BY 102` is the post-mortem: the program
  stopped while running ([chapter 12](12-post-mortems.md)). Its wording is made
  up; the manual says what it reported, not how.
- `lz: alarm: ...` is `lz` saying why, in words. It was not on the page in
  1954.

`lz` types the page first and then anything else it has to say, so a trace
or a reason comes after the values. When there is an error or an alarm, `lz`
exits with a failure.

A tape followed by prose, not by a page, is a fragment, and is not run.

Every sitting in the tutorial is run by `cargo test`, and the test fails if
what `lz` writes differs from the page shown here by one character.

## One rule to learn at once

Two of the rules will surprise anyone who knows a programming language from
after 1954, so learn them now. A blank between two digits is a multiplication
sign: `x = 2 3,` above is 6. And a slash does not divide by the next thing; it
moves what follows it below the line until the next slash moves it back, so
`a/b/c` is ac/b ([chapter 3](03-the-slash.md)).

## The chapters

1. Introduction
2. [Equations](02-equations.md)
3. [The slash](03-the-slash.md)
4. [Numbers](04-numbers.md)
5. [Exponents and functions](05-exponents-and-functions.md)
6. [PRINT](06-print.md)
7. [Jumps and equation numbers](07-jumps.md)
8. [Plus zero and minus zero](08-zero.md)
9. [Superscript variables and indices](09-superscript-variables.md)
10. [The drum](10-the-drum.md)
11. [Differential equations](11-differential-equations.md)
12. [Post-mortems and putting a tape right](12-post-mortems.md)
13. [The temporary rules of 1954](13-temporary-rules.md)
14. [Worked examples](14-worked-examples.md)
15. [What became of it](15-what-became-of-it.md)
16. [Reference](16-reference.md)

Next: [Equations](02-equations.md)
