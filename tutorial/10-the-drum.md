# 10. The drum

## Core and drum

Whirlwind had two kinds of storage: magnetic cores, "MS", fast and small, and a
magnetic drum, large and slow. Letters and superscript variables live in core.
When 250 superscript variables are not enough, more can go on the drum (p.
15).

## ASSIGN

"The instruction ASSIGN followed by a lower case letter followed by a vertical
slash followed by an upper case integer between 1 and 1023 inclusive followed
by a comma causes space on the drum to be selected for variables. For
example, ASSIGN a|1000, automatically reserves space for variables a|1
through a|1000 inclusive to the drum and takes up in the storage table for
variables in MS the space of only one ordinary superscript variable. Once this
assignment has been made, the superscript variables in question may be
treated as ordinary superscript variables."

```lzr
ASSIGN g|4,
g|1 = 2, g|2 = 4, g|3 = 6, g|4 = 0,
PRINT g|1, g|2, g|3, g|4.
STOP
```
```page

g|1 = +.20000000+01     g|2 = +.40000000+01     g|3 = +.60000000+01     g|4 = +.00000000+00
```

ASSIGN is done while the tape is translated, not when it is reached: it is a
statement about storage, not an instruction.

## A list of values: x|N

"Exactly the same thing may be accomplished by writing g|N = 2,4,6,0, Any
lower case letter may be used in this way but only the upper case letter N may
follow the vertical slash. Each value, including the last, must be followed by
a comma." (p. 16)

```lzr
g|N = 2, 4, 6, 0,
PRINT g|1, g|2, g|3, g|4.
STOP
```
```page

g|1 = +.20000000+01     g|2 = +.40000000+01     g|3 = +.60000000+01     g|4 = +.00000000+00
```

Values that go up in even steps can be written with the step in parentheses:
"one writes after the equals sign, the value of a|1, then the increment in
parentheses, then the value up to which one is to go using this increment, and
so on". The manual's example:

```lzr
a|N = 1.0 (.5) 2. (0.25) 2.5 (1) 4.5,
PRINT a|1, a|2, a|3, a|4, a|5, a|6, a|7.
STOP
```
```page

a|1 = +.10000000+01     a|2 = +.15000000+01     a|3 = +.20000000+01     a|4 = +.22500000+01     a|5 = +.25000000+01     a|6 = +.35000000+01
a|7 = +.45000000+01
```

"setting a|1 = 1, a|2 = 1.5, a|3 = 2, a|4 = 2.25, a|5 = 2.5, a|6 = 3.5, a|7 =
4.5". And the integers 1 to 10, which chapter 9's interpolation needs:

```lzr
k|N = 1 (1) 10,
PRINT k|1, k|5, k|10.
STOP
```
```page

k|1 = +.10000000+01     k|5 = +.50000000+01     k|10 = +.10000000+02
```

This is the form the manual calls x|N. Its Temporary Rules say it was not yet
working in January 1954 (chapter 13).

## The cost

"It should be noted that the operations multiplication, addition, etc.,
involving variables on the drum will in general take about 30 milliseconds
longer per variable on the drum than the same operations involving MS
variables." (p. 16) Thirty milliseconds is about the time the drum takes to go
round. `lz --stats` counts the drum accesses:

```lzr --stats
a|N = 1 (1) 100,
s = 0,
n = 0,
1 n = n + 1,
s = s + a|n,
e = n - 99.5,
CP 1,
PRINT s.
STOP
```
```page

s = +.50500000+04
equations carried out      405
blocks brought from drum   302
interpretive orders        902
floating-point operations  300
function subroutine calls  0
drum variable accesses     100  (about 30 ms each, E-364 p. 16: 3.0 s)
integration steps          0
values printed             1
```

A hundred additions from the drum: three seconds, on that figure alone. "If
variables on the drum are used sufficiently often in a program to make a
significant difference in the computer time used, and if there is space for
the variables in MS then the variables should be assigned, or re-assigned to
MS", by copying them into core:

```lzr
y|N = 1 (1) 5,
n = 0,
1 n = n + 1,
f|n = y|n,
e = n - 4.5,
CP 1,
PRINT f|1, f|5.
STOP
```
```page

f|1 = +.10000000+01     f|5 = +.50000000+01
```

## How big the drum is

"There is a maximum of 3072 (which may be increased at a later date) but the
drum is shared by the equations and the drum variables of a program so that in
no case could 3072 drum variables actually be used." How much of the drum an
equation took is not known. `lz` guesses one variable's worth for every two
orders of its translation, and says so when it runs out.

## Data on a separate tape

"It may happen, for example, that a fairly large amount of data will be
processed in several ways at several different times. Rather than duplicate
the long data tape onto the processing program tape in each instance one may
simply place a stop character at the end of the data tape." (p. 9) The
computer stopped, the operator put in the program tape, and pressed restart.

`lz` takes several tapes and reads them in turn:

```bash
lz data.lzr program.lzr
```
