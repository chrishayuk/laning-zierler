# 12. Post-mortems

## The alarm

"Certain automatic post-mortem features have been included but this part of
the computer is essentially still in the design stage." (p. 19)

"If an arithmetic alarm occurs, the programmer will be told the number of the
equation in which the alarm occurred and the number of the equation that
preceded the alarm. It should be noted here that equations not assigned
numbers by the programmer are automatically assigned numbers from 101 to 200
by the computer in the order in which they are written." (p. 20)

```lzr
a = 2,
b = a - 2,
c = a/b,
PRINT c.
STOP
```
```page

ALARM  EQUATION 103  PRECEDED BY 102
lz: alarm: division by zero
```

The page says only the numbers: equation 103, `c = a/b`, after equation 102.
What Whirlwind typed is not shown in the manual, so the wording of that line
is made up. The line after it, beginning `lz:`, is not on the page at all: it
is `lz` telling you why, on standard error.

## Equation 100

"The programmer may obtain the values of any variables he desires after an
arithmetic alarm by writing the appropriate PRINT instruction as equation
100."

```lzr
x = 1,
n = 0,
1 x = 2x,
n = n + 1,
SP 1,
100 PRINT n, x.
STOP
```
```page

ALARM  EQUATION 1  PRECEDED BY 103

n = +.62000000+02       x = +.46116860+19
lz: alarm: a result exceeds 2^63 - 2^39
```

Doubling 1 goes past 2⁶³ - 2³⁹ on the 63rd time. Equation 100 is reached only
after an alarm, or if the program jumps to it; in its place in the program it
is an ordinary PRINT. Knuth and Trabb Pardo: "Another innovation, designed to
help debugging, was to execute statement number 100 after any arithmetic error
message, if 100 was a PRINT statement."

## What causes an alarm

| | |
|---|---|
| a result beyond 2⁶³ - 2³⁹ | the manual's "alarm" (p. 6) |
| division by zero | |
| a function outside its range | √ of a negative number, a logarithm of 0, sin⁻¹ 2 |
| a bad index | negative, between 0 and ½, or a computed zero (chapter 9) |
| storage exceeded | the 251st superscript variable, while running |

```lzr
x = 2,
y = F^5(x),
STOP
```
```page

ALARM  EQUATION 102  PRECEDED BY 101
lz: alarm: F⁵: sine⁻¹ of a number beyond 1
```

## Storage

"Roughly speaking, if a program exceeds storage in any one of several possible
ways the computer will tell us where and how storage was exceeded." (pp.
19-20) Most of those ways are found while the tape is translated: more than
100 numbered or unnumbered equations, two equations with one number, too many
superscript variables named.

```lzr
1 a = 1,
1 b = 2,
STOP
```
```page
<stdin>:2:1: error: two equations are numbered 1
```

## "With a single exception"

The summer session notes of 1954 explain why the post-mortems were still in
the design stage: "Heretofore there has been very little need for elaborate
post-mortem facilities, because, with a single exception, all programs coded
in the Laning-Zierler system have been completed successfully on their first
run."
