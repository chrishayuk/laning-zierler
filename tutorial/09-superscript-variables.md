# 9. Superscript variables and indices

## More variables

"Subscript numbers are not available on the standard Flexowriter so additional
symbols for variables are obtained as follows: Any lower case letter followed
by a vertical slash followed by an upper case integer between 0 and 1023
inclusive may be used as a variable in precisely the same way the lower case
letters alone have been used above. We shall call such variables 'superscript
variables.'" (p. 13)

The number was typed in upper case, as a superscript: x|⁵. Here it may be
typed `x|⁵` or `x|5`.

```lzr
x|5 = 2,
a|2 = x|5 + 1,
y = a|2^2,
PRINT x|5, a|2, y.
STOP
```
```page

x|5 = +.20000000+01     a|2 = +.30000000+01     y = +.90000000+01
```

`a|2^2` is a|2 squared: the 2 after the slash is the superscript and the `^2`
is an exponent. `x|5` and `x` are different variables.

## A letter as the index

"Any lower case letter may be used as a variable index for superscript
variables in the combination lower case letter followed by vertical slash,
followed by lower case letter. For example, when the expression x|n is
encountered, the current value of n is taken as the index: if n = 6 currently
then x|n is at this time treated as x|6."

```lzr
n = 0,
1 n = n + 1,
s|n = n^2,
e = n - 4.5,
CP 1,
i = 3,
PRINT s|1, s|2, s|3, s|4, s|i.
STOP
```
```page

s|1 = +.10000000+01     s|2 = +.40000000+01     s|3 = +.90000000+01     s|4 = +.16000000+02     s|3 = +.90000000+01
```

`PRINT s|i.` types `s|3`, the variable it means now (p. 14).

## Rounding

"If, in the expression x|n, n has a non-integral value greater than 0.5, the
computer rounds n off to the nearest integer." (p. 13) "The roundoff property
can often be used to advantage", for instance in interpolation:

```lzr
k|1 = 10, k|2 = 20, k|3 = 30,
j = 1.4, a = k|j,
j = 1.6, b = k|j,
j = 2.5, c = k|j,
PRINT a, b, c.
STOP
```
```page

a = +.10000000+02       b = +.20000000+02       c = +.30000000+02
```

`examples/1954-interpolation.lzr` is the manual's interpolation program, which
rounds an index to find the place in a table.

## Which values are allowed

"Negative values of n are not permitted in this connection, nor are values
between zero and one half inclusive, with the exception that the value zero is
permissible if assigned by the specific equation n = 0,."

```lzr
n = 0,
a|n = 5,
PRINT a|0.
STOP
```
```page

a|0 = +.50000000+01
```

```lzr
n = .3,
a|n = 5,
STOP
```
```page

ALARM  EQUATION 102  PRECEDED BY 101
lz: alarm: n is +.30000001+00: values between zero and one half inclusive are not permitted as a superscript
```

And a zero that comes out of arithmetic is not "assigned by the specific
equation n = 0". Here it is -0 (chapter 8), and so it is not allowed:

```lzr
n = 1,
n = n - 1,
a|n = 5,
STOP
```
```page

ALARM  EQUATION 103  PRECEDED BY 102
lz: alarm: n is -0, a zero computed by arithmetic: as a superscript zero is permissible only if assigned by the specific equation n = 0,
```

With `--zero plus` the same zero is +0 and is let through:

```lzr --zero plus
n = 1,
n = n - 1,
a|n = 5,
PRINT a|0.
STOP
```
```page

a|0 = +.50000000+01
```

This is what happens to Knuth's TPK program in chapter 14.

## How many

"Superscript variables are ordinarily assigned to MS (magnetic core storage)
and there is room for a total of 250 such variables." (p. 15) The 251st is a
post-mortem:

```lzr
n = 0,
1 n = n + 1,
x|n = n,
SP 1,
100 PRINT n.
STOP
```
```page

ALARM  EQUATION 102  PRECEDED BY 1

n = +.25100000+03
lz: alarm: storage exceeded: x|251 would be superscript variable number 251 in MS, and there is room for 250
```

Each differential equation takes two of the 250 (chapter 11), and each group
of variables on the drum takes one (chapter 10).
