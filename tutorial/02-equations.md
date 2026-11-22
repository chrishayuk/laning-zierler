# 2. Equations

## A letter, an equals sign, an expression, a comma

"All of the lower case letters are available as variables, and equations of
the following form are used: x = ... ," (p. 2). The right-hand side is worked
out and the number is stored in the letter on the left. "Equations are carried
out in the order in which they are written."

```lzr
a = 5,
b = a + 1,
c = ab,
PRINT a, b, c.
STOP
```
```page

a = +.50000000+01       b = +.60000000+01       c = +.30000000+02
```

`ab` is a times b: letters written next to each other are multiplied, as on
paper. Every variable is one letter, so there is nothing else `ab` could mean.

Because the right side is worked out first, an equation may use the letter it
gives a value to:

```lzr
n = 10,
w = 3,
n = n + 2,
w = - w,
PRINT n, w.
STOP
```
```page

n = +.12000000+02       w = -.30000000+01
```

"The first of these adds the number 2 to the contents of the storage assigned
to the variable n ... the second causes the current value of w to be replaced
by its negative." (pp. 2-3)

## A letter must have a value first

"It should be observed that a letter variable must appear on the left hand side
of an equation so that it may be assigned a definite value before it appears
on the right hand side of an equation. ... a program beginning n = m - 1, m =
1, would be incorrect since m would have no preassigned value." (p. 3)

The system did not check this. Here a letter never given a value reads as
zero, and `lz --check` says what the manual says:

```lzr --check
n = m - 1,
m = 1,
PRINT n.
STOP
```
```page
<stdin>:1:5: warning: m is used here before the first equation that gives it a value ("n = m - 1, m = 1, would be incorrect", p. 3)
```

## The tape is a stream of characters

An equation ends with its comma, not with the end of a line. Spaces, tabs,
back spaces and carriage returns are ignored (p. 9), so one line may hold
several equations, and one equation may run over several lines:

```lzr
a = 1, b = 2, c = 3,
d = a + b
      + c,
PRINT d.
STOP
```
```page

d = +.60000000+01
```

The one exception is a blank between two digits, which is a multiplication
sign. That is chapter 4.

## STOP

"The instruction STOP must be placed at the end of every program." (p. 9) It
may appear only there:

```lzr
a = 1,
PRINT a.
```
```page
<stdin>:2:9: error: the instruction STOP must be placed at the end of every program
```

```lzr
STOP
a = 1,
```
```page
<stdin>:1:1: error: STOP may appear only at the end of a program
```

## Upper case is for instructions

The variables are the 26 lower-case letters. Upper-case letters are for the
instructions (`PRINT`, `STOP`, `SP`, `CP`, `SR`, `CR`, `ASSIGN`), for `F`
(functions, chapter 5), `D` (derivatives, chapter 11) and `N` (lists, chapter
10). Anything else is not understood:

```lzr
X = 1,
STOP
```
```page
<stdin>:1:1: error: "X" is not an instruction: the instructions are PRINT, SP, CP, SR, CR, ASSIGN, D and STOP; variables are lower-case letters
```
