# 4. Numbers

## Writing a number

```lzr
a = 0.012, b = 012, c = .012, d = 48, e = 321,
PRINT a, b, c, d, e.
STOP
```
```page

a = +.12000000-01       b = +.12000000+02       c = +.12000000-01       d = +.48000000+02       e = +.32100000+03
```

"No standard form is assumed for numerical quantities by the computer. Thus it
interprets correctly such quantities as 0.012, 012, .012, 48, 321 etc.,
without ambiguity." (p. 5)

## A blank between digits is a multiplication sign

```lzr
x = 23,
y = 2 3,
z = 2
3,
PRINT x, y, z.
STOP
```
```page

x = +.23000000+02       y = +.60000000+01       z = +.60000000+01
```

Blanks are ignored, "except when they occur between the digits of a number".
There, "any of these characters inserted between digits is interpreted as a
multiplication sign. Thus x = 23, ... would set x to the value twenty-three
while x = 2 3, (i.e., 2 followed by space followed by 3) would cause x to
assume the value 6." (pp. 9-10)

A carriage return is a blank too, so `z` is 6. The manual warns that such a
blank is not always visible on the typed copy: "If the combination space,
back-space ... is inserted between digits, the combination ... would be
interpreted as a multiplication sign." (p. 10)

A multiplication sign can also be written. The Flexowriters at the Digital
Computer Laboratory had a colon with its lower dot filed off, giving `·`, and
the colon itself works (p. 10). Here `*` is accepted as well.

```lzr
a = 5·10^4,
b = 5:10^4,
c = 5 10^4,
d = 5*10^4,
PRINT a, b, c, d.
STOP
```
```page

a = +.50000000+05       b = +.50000000+05       c = +.50000000+05       d = +.50000000+05
```

## The 24, 6 number system

"Numbers are stored and arithmetic operations are carried out by the computer
in the 24, 6 number system. In this system numbers appear in the machine
essentially in the form a · 2^b where a is a signed 24 place binary number
with 1/2 ≤ a < 1 or a = 0 and b is a signed 6 place binary number. Thus, it is
useless to supply more than 8 significant decimal figures" (p. 5).

24 binary places are a little over seven decimal digits. A tenth is not exact
in binary, so adding it up shows the error:

```lzr
x = 0,
n = 0,
1 x = x + .1,
n = n + 1,
e = n - 9.5,
CP 1,
PRINT x.
STOP
```
```page

x = +.10000001+01
```

Ten tenths are 1.0000001. (`CP` is [chapter 7](07-jumps.md).) Another example: 1/3, which
the system keeps as the nearest 24-bit fraction, .333333343...

```lzr
a = 1/3,
b = 3a,
c = 6.3,
PRINT a, b, c.
STOP
```
```page

a = +.33333334+00       b = +.10000000+01       c = +.63000001+01
```

## How a number is typed

`PRINT` types a sign, a point, eight digits, and the power of ten: `+.50000000+01`
is .5 · 10¹. Eight digits is one more than 24 bits can really give, so the
eighth digit is often noise: 6.3 is kept as 6.30000019... and typed
`.63000001+01`.

The digits are cut off, not rounded. The manual's own examples show it: it
prints .05 as `+.50000000-01` (p. 6), and the 24-bit number nearest .05 is
.0500000007..., which rounding would type as `.50000001`. So here they are
truncated.

## How large

"If numbers, including, of course, numbers produced as the result of
computations, exceed 2⁶³ - 2³⁹ (or, roughly, 9.2233995 [sic] · 10¹⁸) in
absolute value, an alarm will result." (p. 6)

The decimal is a slip in the manual. 2⁶³ - 2³⁹ is 9223371487098961920, or
9.2233715 · 10¹⁸: the last three digits on the page should be 715. Nothing else
depends on it. The [README](../README.md#what-the-tests-found) lists it under
"What the tests found". The alarm itself is [chapter 12](12-post-mortems.md).

```lzr
a = 4000000000000000000,
b = 2a,
c = 3a,
STOP
```
```page

ALARM  EQUATION 103  PRECEDED BY 102
lz: alarm: a result exceeds 2^63 - 2^39
```

At the other end, the exponent stops at 2⁻⁶³, about 10⁻¹⁹. The manual does not
say what happened below that; here such a number becomes zero.

```lzr
a = .0000000001,
b = a^2,
PRINT a, b.
STOP
```
```page

a = +.10000000-09       b = +.00000000+00
```

## Numbers are read as integers first

"The computer first reads in a number as an integer, placing the decimal point
only after the number is completely read in. This means that no number is
allowed which, without benefit of decimal point, would be interpreted as an
integer exceeding 2⁶³ - 2³⁹." (p. 6)

So the manual's 9223314040000000000 is allowed, but nineteen digits with a
point in the middle are not, even though their value is small:

```lzr
a = 9223314040000000000,
PRINT a.
STOP
```
```page

a = +.92233143+19
```

```lzr
a = 922337203.6854775807,
STOP
```
```page
<stdin>:1:5: error: 922337203.6854775807: read without its point this is an integer greater than 2^63 - 2^39
```

Next: [Exponents and functions](05-exponents-and-functions.md)
