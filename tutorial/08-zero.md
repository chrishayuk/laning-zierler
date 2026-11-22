# 8. Plus zero and minus zero

## The manual's warning

"If n had the value 5 in the above example then e = n - 5 = 0. In this case
the CP may or may not behave like an SP since 0 may appear as +0 or -0. This
ambiguity need cause no difficulty in general." (p. 8)

Whirlwind kept numbers in one's complement, in which zero has two forms: all
noughts, +0, and all ones, -0. CP looks at the sign, so it jumps on -0.

## Where -0 comes from

In one's complement, a number added to its own negative gives all ones: -0.
So does +0 added to -0. Only a zero typed as a constant, or the sum of two +0s,
is +0. Here that is the rule (the README says why):

```lzr
n = 5,
e = n - 5,
f = 0,
g = -f,
h = 0 + 0,
PRINT e, f, g, h.
STOP
```
```page

e = -.00000000+00       f = +.00000000+00       g = -.00000000+00       h = +.00000000+00
```

The sign shows when it is typed.

## The manual's example

With n = 5, the CP jumps, and x = 3ab is never computed:

```lzr
a = 2, b = 1, x = 0,
n = 5,
e = n - 5,
CP 5,
x = 3 ab,
5 y = ax,
PRINT e, x, y.
STOP
```
```page

e = -.00000000+00       x = +.00000000+00       y = +.00000000+00
```

`lz --zero plus` makes every zero +0, as a reader of today might expect. Then
the CP does not jump:

```lzr --zero plus
a = 2, b = 1, x = 0,
n = 5,
e = n - 5,
CP 5,
x = 3 ab,
5 y = ax,
PRINT e, x, y.
STOP
```
```page

e = +.00000000+00       x = +.60000000+01       y = +.12000000+02
```

Which one Whirlwind did depended on how its interpretive routines handled a
zero, and that is not recorded. The manual does not say: it says not to rely
on it.

## The half

"If, for example, it is desired to go thru a certain process starting with
equation 1 exactly 10 times" (chapter 7) the manual subtracts 9.5, not 10, so
the test never meets a zero. That is the whole cure, and every loop in the
manual uses it. A loop that subtracts 10 goes round once too often here:

```lzr
n = 0,
1 n = n + 1,
e = n - 10,
CP 1,
PRINT n.
STOP
```
```page

n = +.11000000+02
```

With n = 10, e is -0, and CP jumps once more. `lz --check` notices a CP right
after a difference of whole numbers:

```lzr --check
n = 0,
1 n = n + 1,
e = n - 10,
CP 1,
PRINT n.
STOP
```
```page
<stdin>:4:1: hint: if e comes out exactly zero, "the CP may or may not behave like an SP since 0 may appear as +0 or -0" (p. 8). The manual's own loops compare with a half: n - 9.5, not n - 10
```

## Testing for zero

To ask "is r zero?" when the sign of a zero cannot be trusted, test something
that cannot be zero, such as |r| - .5 for a whole number r. F¹¹ is the
absolute value:

```lzr
a = 7, b = 7,
e = F^11(a - b) - .5,
CP 3,
PRINT a.
SP 4,
3 PRINT b.
4 STOP
```
```page

b = +.70000000+01
```

`examples/gcd.lzr` uses this.
