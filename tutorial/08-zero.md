# 8. Plus zero and minus zero

## The manual's warning

"If n had the value 5 in the above example then e = n - 5 = 0. In this case
the CP may or may not behave like an SP since 0 may appear as +0 or -0. This
ambiguity need cause no difficulty in general." (p. 8) The example is the
manual's `e = n - 5, CP 5,`, below.

Whirlwind kept numbers in one's complement, in which zero has two forms: all
noughts, +0, and all ones, -0. CP looks at the sign, so it jumps on -0.

## Where -0 comes from

```lzr
n = 5,
e = n - 5,
f = 0,
g = -f,
h = 0 + 0,
p = 0·5,
q = e·3,
r = F^11(e),
PRINT e, f, g, h.
PRINT p, q, r.
STOP
```
```page

e = -.00000000+00       f = +.00000000+00       g = -.00000000+00       h = +.00000000+00

p = +.00000000+00       q = -.00000000+00       r = +.00000000+00
```

In one's complement, a number added to its own negative gives all ones: -0.
So does +0 added to -0. Here that is the rule: the difference of two equal
numbers is -0, and so is any sum with a -0 in it. A product or a quotient
takes its sign from its operands, as any product does, so 0·5 is +0 and
(-0)·3 is -0. Negating flips the sign, zero or not. And the absolute value,
F¹¹, is always +0. The constant 0, and anything made only of +0s, is +0.

The sign shows when it is typed. This rule is the reconstruction's inference
from the machine and from the manual's warning on p. 8, not something the
manual or any listing says ([README](../README.md#zero)).

## The manual's example

With n = 5, the CP jumps, and x = 3ab is never computed. (In the program on
p. 8, `x = 3 ab` has no comma; the prose below it writes "x = 3 ab,", and the
rules require one, so it is put in here.)

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
equation 1 exactly 10 times" (p. 8, and [chapter 7](07-jumps.md)) the manual subtracts 9.5, not 10, so
the test never meets a zero. That is the whole cure. Every loop in the manual
keeps its test away from zero, most of them with a half like this; the
iterative cosine of p. 11 ([chapter 14](14-worked-examples.md)) does it by
parity instead, testing `1 - y` with `y` even. A loop that subtracts 10 goes
round once too often here:

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

[`examples/gcd.lzr`](../examples/gcd.lzr) uses this.

Next: [Superscript variables and indices](09-superscript-variables.md)
