# 6. PRINT

## Letters and a period

```lzr
x = 1, y = 2,
PRINT x,y.
PRINT xy.
STOP
```
```page

x = +.10000000+01       y = +.20000000+01

x = +.10000000+01       y = +.20000000+01
```

"The current value of any number of letter variables may be printed out by
inserting the instruction PRINT followed by the desired letters, followed by
a period. No computation may be performed during a print instruction (e.g.,
PRINT 3a is not admissible) and commas may, but need not, be inserted between
letters. Thus, PRINT x,y. and PRINT xy. would both cause the current values of
x and y to be printed out." (p. 6)

```lzr
a = 1,
PRINT 3a.
STOP
```
```page
<stdin>:2:7: error: no computation may be performed in a PRINT instruction: only letters, commas and a period, not "3"
```

## The form of a value

"If a = 5 currently the instruction PRINT a. would cause a = +.50000000 + 01
to be printed out. The second number, i.e., the +01, is interpreted as the
power of 10 to be multiplied by the first number. Thus, if a = .05 we should
have a = +.50000000-01 and if a = -π we should have a = -.31415927 + 01."
(p. 6)

```lzr
a = 5,
b = .05,
c = -3.1415927,
PRINT a, b, c.
STOP
```
```page

a = +.50000000+01       b = +.50000000-01       c = -.31415927+01
```

The manual's typesetting puts spaces around the sign of the exponent in two
places and not in a third. What the delayed printer did is not known; here
there are none.

## Carriage returns and lines

"When a PRINT instruction is performed the first and last characters to
appear as output are carriage returns. If more than 6 variables are called
for on a single PRINT instruction, 6 will be placed on each line, except
possibly for the last, of course, and there will be no line space between
lines" (pp. 6-7).

```lzr
a = 1, b = 2, c = 3, d = 4, e = 5, f = 6, g = 7, h = 8, i = 9,
PRINT a, b, c, d, e, f, g, h.
STOP
```
```page

a = +.10000000+01       b = +.20000000+01       c = +.30000000+01       d = +.40000000+01       e = +.50000000+01       f = +.60000000+01
g = +.70000000+01       h = +.80000000+01
```

A PRINT begins and ends with a carriage return, so two PRINTs one after the
other give two carriage returns between them, which is a line space. That is
the manual's second example: "while the instructions PRINT a,b,c. PRINT d,e,f.
PRINT g,h,i. will produce" three lines with spaces between them.

```lzr
a = 1, b = 2, c = 3, d = 4, e = 5, f = 6, g = 7, h = 8, i = 9,
PRINT a,b,c. PRINT d,e,f. PRINT g,h,i.
STOP
```
```page

a = +.10000000+01       b = +.20000000+01       c = +.30000000+01

d = +.40000000+01       e = +.50000000+01       f = +.60000000+01

g = +.70000000+01       h = +.80000000+01       i = +.90000000+01
```

The page begins with a blank line for the same reason: the first character of
the first PRINT is a carriage return. The columns here are every 24
characters; the manual's are not measured.

## What a PRINT may be

"PRINT instructions may be numbered" (section 17, rule 7, p. 20), so a jump
can go to one. "Every PRINT instruction must end with a period" (rule 4):

```lzr
a = 1,
PRINT a,
STOP
```
```page
<stdin>:2:8: error: every PRINT instruction must end with a period, not a comma
```

Superscript variables are printed with their superscripts
([chapter 9](09-superscript-variables.md)).

Next: [Jumps and equation numbers](07-jumps.md)
