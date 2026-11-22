# 16. Reference

A page number alone is E-364's; the other sources are listed in
[chapter 1](01-introduction.md#the-sources).

## A tape

A tape is a stream of characters. An equation ends with a comma, a PRINT with
a period, and STOP ends the program. Spaces, tabs, back spaces and carriage
returns are ignored, except between digits, where they mean times (pp. 9-10).
The nullify character is ignored everywhere, even inside a number, except
inside the name of a jump or STOP (rule 2 below), where it makes the
instruction unreadable. `#` begins a note, to the end of the line (not
original).

## Statements

| | |
|---|---|
| `[n] x = expression,` | an equation (p. 2) |
| `[n] D x = expression,` | dx/dt; `t` is the independent variable and `h` the step (p. 18) |
| `[n] PRINT a, b, x\|3, y\|i.` | type the current values; commas optional (p. 6) |
| `SP n,` | go to equation n (p. 8) |
| `CP n,` | go to n if the last computed quantity is negative |
| `SR n,` | carry out equation n, then come back (p. 9) |
| `CR n,` | the same if the last computed quantity is negative |
| `SP u,` etc. | the equation number is the value of u (p. 15) |
| `ASSIGN a\|1000,` | a\|1 to a\|1000 on the drum (p. 15) |
| `a\|N = 2, 4, 6, 0,` | values for a\|1, a\|2, ... on the drum (p. 16) |
| `a\|N = 1 (.5) 2 (1) 4,` | the same, with increments in parentheses |
| `[n] STOP` | the end; must be the last statement (p. 9) |

`[n]` is an equation number from 1 to 100. Jumps may not be numbered.
Unnumbered equations are numbered 101 to 200 in order (p. 20).

## Expressions

| | |
|---|---|
| `+` `-` | separate terms (p. 4) |
| `/` | what follows, to the end of the term or the next slash, is below the line: `a/b/c` is ac/b (pp. 4-5) |
| `ab`, `a·b`, `a:b`, `a*b`, `2 3` | times |
| `x²`, `x^2`, `x⁻²`, `x^-2` | a whole-number power (p. 5) |
| `( )` | grouping; at most four open at once (p. 3) |
| `F^k(x)`, `Fᵏ(x)` | function subroutine k (p. 12) |
| `x\|5`, `x\|⁵` | a superscript variable, 0 to 1023 (p. 13) |
| `x\|n` | the superscript is the value of n, rounded (p. 13) |

## Functions

| | | | | | |
|---|---|---|---|---|---|
| F¹ | √ | F⁹ | csc | F¹⁷ | tanh |
| F² | sin | F¹⁰ | cot | F¹⁸ | sech |
| F³ | cos | F¹¹ | \|x\| | F¹⁹ | csch |
| F⁴ | tan | F¹² | signum | F²⁰ | coth |
| F⁵ | sin⁻¹ | F¹³ | eˣ | F²¹ | log₁₀ |
| F⁶ | cos⁻¹ | F¹⁴ | e⁻ˣ | F²² | log₂ |
| F⁷ | tan⁻¹ | F¹⁵ | sinh | F²³ | logₑ |
| F⁸ | sec | F¹⁶ | cosh | | |

## Numbers

24 binary digits and a binary exponent from -63 to 63; at most 2⁶³ - 2³⁹ in
size (pp. 5-6). A number as typed, read without its point, must not exceed
that either. Values are typed as a sign, a point, eight digits (cut off, not
rounded) and the power of ten: `+.50000000+01`. Zero has a sign: `-.00000000+00`.

## Limits

| | |
|---|---|
| numbered equations | 100 |
| unnumbered equations | 100 |
| open parentheses | 4 |
| superscript variables in core | 250, less 2 for each D equation, less 1 for each group on the drum |
| superscript | 0 to 1023 |
| drum | 3072 variables, shared with the equations |
| function subroutines | 23 |

## Zero and indices

| | |
|---|---|
| a difference of equal numbers, or a sum with -0 in it | -0 |
| the constant 0, a sum of +0s, \|x\| | +0 |
| a product or quotient | the sign of the operands, as for any number |
| `CP`, `CR` | jump when the last computed quantity is negative, -0 included (p. 8) |
| an index `x\|n` | n rounded to the nearest whole number, a half rounded up (p. 13) |
| an index that is not allowed | negative, between 0 and ½ inclusive, or a zero not set by `n = 0,` (p. 13) |

The sign of a computed zero, and the way a zero set by `n = 0,` is told from a
computed one, are the reconstruction's inference ([chapter 8](08-zero.md)).
`--zero plus` makes every zero +0.

## Post-mortems

| | |
|---|---|
| a result beyond 2⁶³ - 2³⁹ | alarm (p. 6) |
| division by zero | alarm |
| a function outside its range | alarm |
| an index that is not allowed | alarm |
| the 251st superscript variable | alarm: storage exceeded |
| equation 100 | if it is a PRINT, carried out after an alarm (p. 20) |
| equations without numbers | numbered 101 to 200 in order (p. 20) |

An alarm is typed `ALARM  EQUATION n  PRECEDED BY m` (wording made up).
[Chapter 12](12-post-mortems.md) has them all.

## The command line

| | |
|---|---|
| `lz tape ...` | translate the tapes, read in turn, and run; type the page |
| `lz` | read the tape from standard input |
| `--translate` | each equation's number and its block of orders; nothing is run |
| `--check` | errors, warnings and hints; nothing is run |
| `--trace` | each equation as it is carried out, after the page |
| `--stats` | what the run cost, after the page |
| `--zero ones` / `--zero plus` | how a zero gets its sign (ones is the default) |
| `--temporary-rules` | the restrictions of January 1954 (p. 21) |
| `--limit N` | stop after N equations; 5,000,000 unless told |

`lz` exits with a failure when the tape cannot be translated, when there is an
alarm, or when the limit is reached.

## What `lz` says

An **error** stops translation, and nothing is run. A **warning** is something
the manual says is wrong but that does not stop the tape. A **hint** is
something the manual says to be careful of. Warnings and hints come only from
`lz --check`. One tape with many things wrong:

```lzr --check
1 a = 5,
1 b = m + 2,
x = a(b(a(b(a(b))))),
y = F^24(a),
4 SP 2,
SP 9,
PRINT 3a.
X = 1,
n = 0,
2 n = n + 1,
e = n - 10,
CP 2,
q = 2^1.5,
```
```page
<stdin>:2:1: error: two equations are numbered 1
<stdin>:2:7: warning: m is never given a value: a letter must appear on the left of an equation before it appears on the right
<stdin>:3:14: error: no more than four parentheses may be open in a single equation at any one time
<stdin>:4:6: error: there are 23 function subroutines, F¹ to F²³; there is no F^24
<stdin>:5:3: error: an SP may not itself be assigned an equation number
<stdin>:6:1: error: SP 9: there is no equation numbered 9
<stdin>:7:7: error: no computation may be performed in a PRINT instruction: only letters, commas and a period, not "3"
<stdin>:8:1: error: "X" is not an instruction: the instructions are PRINT, SP, CP, SR, CR, ASSIGN, D and STOP; variables are lower-case letters
<stdin>:12:1: hint: if e comes out exactly zero, "the CP may or may not behave like an SP since 0 may appear as +0 or -0" (p. 8). The manual's own loops compare with a half: n - 9.5, not n - 10
<stdin>:13:6: warning: this is read as the power 1 times .5: exponents are whole numbers ("positive (unsigned) and negative integral exponents", p. 5)
```

The tape has no STOP either. That is reported only when nothing else is wrong.

Errors, when the tape is read:

| message | why |
|---|---|
| expected a comma to end the equation, not ... | rule 1; the mistake is often a line before |
| every PRINT instruction must end with a period | rule 4 |
| no computation may be performed in a PRINT instruction | p. 6 |
| no more than four parentheses may be open ... | rule 5 |
| an SP may not itself be assigned an equation number | rule 6 |
| the instruction SP must be terminated by a comma | rule 6 |
| equations are numbered 1 to 100 | p. 7 |
| two equations are numbered n | |
| SP n: there is no equation numbered n | |
| there are 23 function subroutines | p. 12 |
| a superscript is an integer from 0 to 1023 | p. 13 |
| ... is not an instruction | upper case is for instructions |
| two signs together | put the second term in parentheses |
| t is the independent variable and h its increment | neither may follow D (p. 18) |
| each value, including the last, must be followed by a comma | x\|N lists (p. 16) |
| read without its point this is an integer greater than 2^63 - 2^39 | p. 6 |
| the instruction STOP must be placed at the end of every program | p. 9 |
| STOP may appear only at the end of a program | rule 8 |
| storage exceeded | too many superscript variables, or too much on the drum (pp. 15-17) |
| a maximum of 100 numbered equations and 100 unnumbered equations ... | rule 11 |
| "The x\|N is not yet in operation" | `--temporary-rules` (p. 21) |
| "An SR or CR instruction may not refer to an equation that makes use of a subroutine" | `--temporary-rules` (p. 21) |

Alarms, while the program runs (the `lz:` line):

| message | |
|---|---|
| a result exceeds 2^63 - 2^39 | p. 6 |
| division by zero | |
| F¹: the square root of a negative number | and the other functions' ranges |
| a logarithm of a number that is not positive | |
| n is negative / between zero and one half / -0, a zero computed by arithmetic | an index (p. 13) |
| SP n: there is no equation numbered n | a jump to the value of a letter |
| storage exceeded: x\|n would be superscript variable number 251 | p. 15 |

Warnings and hints, from `lz --check`:

| | |
|---|---|
| warning: m is never given a value / is used here before ... | p. 3 |
| warning: a is computed from the solution outside the D equations ... | p. 19 |
| warning: t / h is never given a value | p. 18 |
| warning: this is read as the power 1 times .5 | p. 5 |
| hint: if e comes out exactly zero, the CP may or may not ... | p. 8 |
| hint: i is used as an index and counted down by arithmetic ... | p. 13 |

## The rules of section 17

As the manual gives them (p. 20):

1. Every equation must end with a comma.
2. The nullify character is always ignored except when it occurs between the
   S and P of SP (similarly for SR, CR, CP, STOP).
3. Except when they occur between the digits of a number or in SP, etc., the
   following characters are ignored: space, back space, tabulate and carriage
   return. Any of these characters inserted between numbers is interpreted as
   a multiplication sign.
4. Every PRINT instruction must end with a period.
5. No more than 4 open parentheses may be written in a single equation before
   one is closed.
6. The instructions SP, CP, SR, CR must be terminated by a comma and may not be
   numbered.
7. PRINT instructions may be numbered.
8. The instruction STOP may be numbered and may appear only at the end of a
   program.
9. The instruction ASSIGN must be terminated by a comma.
10. Each assignment of a group of variables to the drum (by either the ASSIGN or
    the x|N type of instruction) uses up the space ordinarily occupied by one
    superscript variable in the table for superscript variables in MS. With
    this understanding a total of 250 superscript variables may be assigned to
    MS.
11. A maximum of 100 numbered equations and 100 unnumbered equations may be
    used in any one problem.

## Characters a keyboard does not have

| Flexowriter | type |
|---|---|
| upper-case digits (exponents) | `²` or `^2` |
| upper-case minus | `⁻` or `^-` |
| `·` (a colon with a dot filed off) | `·`, `:` or `*` |
| superscript after a vertical slash | `x\|⁵` or `x\|5` |
| nullify | DEL (0x7F) |

Back to the [introduction](01-introduction.md), or on to the
[examples](../examples/).
