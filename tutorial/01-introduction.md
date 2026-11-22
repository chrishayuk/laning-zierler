# 1. Introduction

## What this was

In 1952 J. Halcombe Laning Jr., of the MIT Instrumentation Laboratory, wrote a
program for the Whirlwind I computer that read mathematical equations, typed
in ordinary notation, and did what they said. By May 1953 he and Neal Zierler
had made it into a system other people could use, and in January 1954 they
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
backspace, carriage return, etc.) that is typed". The paper tape went to
Whirlwind with a request: by the spring of 1954, load the master tape
108-60-30, put your tape in the photoelectric reader, press start. The system
read the whole tape, translated every equation into Whirlwind instructions and
stored them on the magnetic drum, and then carried them out. Its answers went
to magnetic tape, and the "delayed printer" typed them afterwards.

So there is no conversation. You hand over a tape and get back a page.

## Running `lz`

`lz` takes a tape (a text file) and types the page:

```bash
lz examples/1954-first-equations.lzr
```

With no file it reads the tape from standard input. In this tutorial each
sitting is a tape and then the page it gives:

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

The first three lines are the manual's first example (p. 2). `PRINT` typed the
three values; `STOP` ended the program. `y` is -31.500001, not -31.5, and the
reason is chapter 4.

A `#` starts a note that runs to the end of the line. Notes are not part of
the tape; they are for the people reading the file.

## The options

| | |
|---|---|
| `lz tape` | translate and run; type the page |
| `lz data program` | several tapes, read one after another, as a data tape and then a program tape were |
| `lz --translate tape` | show each equation's number and the block of orders it was translated into |
| `lz --check tape` | report problems and warnings without running |
| `lz --trace tape` | show each equation as it is carried out |
| `lz --stats tape` | count what the run cost |
| `lz --zero plus tape` | make every zero +0 (chapter 8) |
| `lz --temporary-rules tape` | the restrictions of January 1954 (chapter 13) |

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
12. [Post-mortems](12-post-mortems.md)
13. [The temporary rules of 1954](13-temporary-rules.md)
14. [Worked examples](14-worked-examples.md)
15. [Reference](15-reference.md)
