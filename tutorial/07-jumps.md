# 7. Jumps and equation numbers

## Numbering an equation

```lzr
3 x = 2,
1 y = 0.5 (x + 2/x),
PRINT x, y.
STOP
```
```page

x = +.20000000+01       y = +.15000000+01
```

"Equations may be numbered with decimal integers between 1 and 100 inclusive.
... The first equation is then identified as equation number 3, the second as
equation number 1. The equation number always precedes the equation to which
it refers." (pp. 7-8) The numbers need not be in order; they are names.

A number in front is always an equation number, never part of the equation:
`5y = ax,` is equation 5, `y = ax`.

## SP: go to

"The effect of the instruction SP followed by an integer from 1 to 100 and a
comma is to 'transfer control' to the equation with the specified number and
to proceed on from that equation." (p. 8)

```lzr
x = 1,
SP 7,
x = 100,
7 PRINT x.
STOP
```
```page

x = +.10000000+01
```

## CP: go to, if negative

"The effect of the instruction CP 5, is the same as that of SP 5, if the last
computed quantity is negative while CP 5, is ignored if the last computed
quantity is positive." (p. 8) The last computed quantity is the value of the last
equation carried out. There is no comparison: to ask whether n is less than
10, compute n - 10 and see whether it is negative.

The manual's own loop, "to go thru a certain process starting with equation 1
exactly 10 times":

```lzr
n = 0,
s = 0,
1 s = s + n,
n = n + 1,
e = n - 9.5,
CP 1,
PRINT n, s.
STOP
```
```page

n = +.10000000+02       s = +.45000000+02
```

"The quantity n is thus used as a counter, and e will then have the successive
values -8.5, -7.5, -6.5, ..., -0.5, +0.5 and we obtain the desired result."
(p. 9) Why 9.5 and not 10 is [chapter 8](08-zero.md).

## SR and CR: one equation, and back

"The instruction SR 5, causes equation 5 to be executed next and then, after
equation 5, the equation or instruction following the SR 5. The instruction
CR 5, bears the same relation to SR 5, that CP 5, does to SP 5." (p. 9)

SR borrows a single equation. It is not a subroutine call in the later sense:
the equation is carried out and control comes straight back.

```lzr
x = 2,
SR 50,
SR 50,
PRINT x.
y = -7,
CR 60,
PRINT y.
SP 70,
50 x = x^2,
60 y = -y,
70 STOP
```
```page

x = +.16000000+02

y = +.70000000+01
```

## Jump instructions have no numbers

"An SP, CP, SR, or CR may not itself be assigned an equation number." (p. 9)

```lzr
x = 1,
4 SP 4,
STOP
```
```page
<stdin>:2:3: error: an SP may not itself be assigned an equation number
```

And a jump must go to an equation that has the number:

```lzr
SP 9,
STOP
```
```page
<stdin>:1:1: error: SP 9: there is no equation numbered 9
```

## The numbers the translator gives

"Equations not assigned numbers by the programmer are automatically assigned
numbers from 101 to 200 by the computer in the order in which they are
written." (p. 20) The numbers matter when something goes wrong
([chapter 12](12-post-mortems.md)). `lz --translate` shows them, beside each
equation's block of orders:

```lzr --translate
a = 5,
SP 3,
a = 0,
3 x = a + 1,
PRINT x.
STOP
```
```page
101  a = 5,
       const +.50000000+01
       store a
     SP 3,
       go to 3
102  a = 0,
       const +.00000000+00
       store a
  3  3 x = a + 1,
       load  a
       const +.10000000+01
       add
       store x
103  PRINT x.
       print x
104  STOP
       stop
```

"A maximum of 100 numbered equations and 100 unnumbered equations may be used
in any one problem." (section 17, rule 11, p. 20)

## A letter as the target

"It is also possible to program such a change by using the following
instruction: SP followed by any lower case letter followed by a comma. If, for
example, the variable u has the value 15, when the instruction SP u, is
reached by the control, then this instruction will have the effect of SP 15,."
(p. 15)

```lzr
u = 12,
SP u,
11 PRINT u.
SP 13,
12 u = u - 1,
SP u,
13 STOP
```
```page

u = +.11000000+02
```

The value is rounded as an index is ([chapter 9](09-superscript-variables.md)).

Next: [Plus zero and minus zero](08-zero.md)
