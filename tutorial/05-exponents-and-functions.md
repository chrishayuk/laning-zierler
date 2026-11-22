# 5. Exponents and functions

## Exponents

```lzr
a = 5²,
b = 5^2,
c = 2^-2,
d = 2⁻²,
e = 7^0,
PRINT a, b, c, d, e.
STOP
```
```page

a = +.25000000+02       b = +.25000000+02       c = +.25000000+00       d = +.25000000+00       e = +.10000000+01
```

"Upper case numbers on the Flexowriter appear as exponents; there is an upper
case minus sign but no upper case plus sign. The computer will handle positive
(unsigned) and negative integral exponents." (p. 5)

A keyboard today has no upper-case digits. Here an exponent may be typed with
superscript characters, `5²`, `a⁻³`, or with `^`: `5^2`, `a^-3`. The two are
the same.

An exponent belongs to the thing just before it: a number, a letter, a
parenthesis or a function. In `2ay^2` only `y` is squared.

```lzr
a = 3,
y = 2,
p = 2ay^2,
q = (2ay)^2,
r = (a - 1)^-2,
PRINT p, q, r.
STOP
```
```page

p = +.24000000+02       q = +.14400000+03       r = +.25000000+00
```

A power is worked out by multiplying, and a negative power is one divided by
that. An exponent is a whole number, and that is not a rule you can break by
mistake so much as one that breaks you quietly. The point and the 5 in
`2^1.5` are lower case, a number of their own, so the tape says 2¹ times .5:

```lzr
x = 2^1.5,
PRINT x.
STOP
```
```page

x = +.10000000+01
```

`lz --check` warns about it:

```lzr --check
x = 2^1.5,
PRINT x.
STOP
```
```page
<stdin>:1:6: warning: this is read as the power 1 times .5: exponents are whole numbers ("positive (unsigned) and negative integral exponents", p. 5)
```

To raise to the power 1.5, use the functions below: `F^13(1.5 F^23(x))` is
e^(1.5 ln x).

The manual's examples of exponents (p. 5), run one after another, are in
[`examples/1954-exponents.lzr`](../examples/1954-exponents.lzr). They do not
finish. (ab³)⁵ is about 10⁻³⁴, too small for the 24, 6 system: here it becomes
zero ([chapter 4](04-numbers.md)), and `(ab³)⁻⁵` divides by it. Even with more
range, 1/10⁻³⁴ would be far beyond 2⁶³ - 2³⁹.

## The function subroutines

"It is clearly desirable to build into the computer subroutines that compute
automatically certain frequently used functions. This process has been begun
for our computer and at present the following functions are available" (p.
12). A function is called by writing its number as the exponent of an
upper-case `F`:

| | | | | | |
|---|---|---|---|---|---|
| F¹ | square root | F⁹ | cosecant | F¹⁷ | tanh |
| F² | sine | F¹⁰ | cotangent | F¹⁸ | sech |
| F³ | cosine | F¹¹ | absolute value | F¹⁹ | csch |
| F⁴ | tangent | F¹² | signum | F²⁰ | coth |
| F⁵ | sine⁻¹ | F¹³ | eˣ | F²¹ | log₁₀ |
| F⁶ | cosine⁻¹ | F¹⁴ | e⁻ˣ | F²² | log₂ |
| F⁷ | tangent⁻¹ | F¹⁵ | sinh | F²³ | logₑ |
| F⁸ | secant | F¹⁶ | cosh | | |

"The sine of x may be obtained by writing simply F²(x) and such an expression
may be treated arithmetically in the same way as a variable in an equation"
(p. 12):

```lzr
y = 4, z = -0.5,
x = 2(F^1(y) F^2(z) + F^11(z)),
PRINT x.
STOP
```
```page

x = -.91770219+00
```

That is 2(√y sin z + |z|), the manual's example. "Computation may be carried
out inside the parentheses containing the argument of a function", and
"functions of functions are permitted" (p. 13):

```lzr
a = -9, z = .5, y = 1,
p = F^1(F^11(a)),
q = F^4(y + F^6(z)),
r = F^23(F^13(2.5)),
PRINT p, q, r.
STOP
```
```page

p = +.30000000+01       q = -.19378145+01       r = +.25000000+01
```

`p` is √|a|, `q` is tan(y + cos⁻¹ z), and `r` is the logarithm of e^2.5.

## The functions are not the originals

The routines of 1954 are not known. Here each function is worked out in double
precision from the 24-bit argument and rounded to 24 bits. A function outside
its range is an alarm:

```lzr
a = -4,
b = F^1(a),
STOP
```
```page

ALARM  EQUATION 102  PRECEDED BY 101
lz: alarm: F¹: the square root of a negative number
```

There are 23 and no more:

```lzr
x = F^24(1),
STOP
```
```page
<stdin>:1:6: error: there are 23 function subroutines, F¹ to F²³; there is no F^24
```

Next: [PRINT](06-print.md)
