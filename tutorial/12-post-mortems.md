# 12. Post-mortems and putting a tape right

## The alarm

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

"If an arithmetic alarm occurs, the programmer will be told the number of the
equation in which the alarm occurred and the number of the equation that
preceded the alarm. It should be noted here that equations not assigned
numbers by the programmer are automatically assigned numbers from 101 to 200
by the computer in the order in which they are written." (p. 20) "Certain
automatic post-mortem features have been included but this part of the
computer is essentially still in the design stage." (p. 19)

The page says only the numbers: equation 103, `c = a/b`, after equation 102.
What Whirlwind typed is not shown in the manual, so the wording of that line
is made up. The line after it, beginning `lz:`, is not on the page at all: it
is `lz` telling you why, on standard error.

## Equation 100

"The programmer may obtain the values of any variables he desires after an
arithmetic alarm by writing the appropriate PRINT instruction as equation
100." (p. 20)

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
message, if 100 was a PRINT statement." (KTP 76, p. 58)

## What causes an alarm

| | |
|---|---|
| a result beyond 2⁶³ - 2³⁹ | the manual's "alarm" (p. 6) |
| division by zero | |
| a function outside its range | √ of a negative number, a logarithm of 0, sin⁻¹ 2 |
| a bad index | negative, between 0 and ½, or a computed zero ([chapter 9](09-superscript-variables.md)) |
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
run." (SS 54, p. 13-4)

## Putting a tape right

Here is a tape with two mistakes in it. It is meant to add up 1/(n - 3)² for n
from 1 to 5, leaving out n = 3. The first mistake is a missing comma:

```lzr
s = 0,
n = 0,
1 n = n + 1,
y = 1/(n - 3)^2,
s = s + y
e = n - 5,
CP 1,
PRINT n, s.
STOP
```
```page
<stdin>:6:3: error: expected a comma to end the equation, not "="
```

The error is a line late. Without its comma, `s + y` runs on into `e`, and
`y e` is y times e, which is a perfectly good expression; it is the equals
sign after it that cannot be read. When the place an error is reported looks
right, look at the line before.

With the comma back, `lz --check` has a hint for the loop:

```lzr --check
s = 0,
n = 0,
1 n = n + 1,
y = 1/(n - 3)^2,
s = s + y,
e = n - 5,
CP 1,
PRINT n, s.
STOP
```
```page
<stdin>:7:1: hint: if e comes out exactly zero, "the CP may or may not behave like an SP since 0 may appear as +0 or -0" (p. 8). The manual's own loops compare with a half: n - 4.5, not n - 5
```

Take the hint, and add an equation 100 to see where the program is if it
stops. `SP 99` steps over it at the end, so that it is typed only after an
alarm:

```lzr
s = 0,
n = 0,
1 n = n + 1,
y = 1/(n - 3)^2,
s = s + y,
e = n - 4.5,
CP 1,
PRINT n, s.
SP 99,
100 PRINT n, s.
99 STOP
```
```page

ALARM  EQUATION 103  PRECEDED BY 1

n = +.30000000+01       s = +.12500000+01
lz: alarm: division by zero
```

Equation 103 is the fourth line, after equation 1, and n is 3: the term that
was to be left out was not. Test for it, with the absolute value so that the
sign of a zero does not matter ([chapter 8](08-zero.md)):

```lzr
s = 0,
n = 0,
1 n = n + 1,
e = F^11(n - 3) - .5,
CP 2,
s = s + 1/(n - 3)^2,
2 e = n - 4.5,
CP 1,
PRINT n, s.
SP 99,
100 PRINT n, s.
99 STOP
```
```page

n = +.50000000+01       s = +.25000000+01
```

¼ + 1 + 1 + ¼ is 2.5.

Next: [The temporary rules of 1954](13-temporary-rules.md)
