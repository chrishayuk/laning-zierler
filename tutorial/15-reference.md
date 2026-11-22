# 15. Reference

Page numbers are E-364's.

## A tape

A tape is a stream of characters. An equation ends with a comma, a PRINT with
a period, and STOP ends the program. Spaces, tabs, back spaces and carriage
returns are ignored, except between digits, where they mean times (p. 9). The
nullify character is always ignored. `#` begins a note, to the end of the line
(not original).

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
size (p. 5-6). A number as typed, read without its point, must not exceed
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

## The rules of section 17

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
