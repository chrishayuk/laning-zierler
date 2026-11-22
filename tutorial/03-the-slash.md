# 3. The slash

## Plus and minus separate terms

"A little reflection will show that an expression such as a + 2b/c + d is
mathematically ambiguous. The computer of course, assigns a particular
interpretation to such an expression. ... a plus or minus sign has the effect
of separating any mathematical expression into two terms to the left and the
right of it." (p. 4)

So in `a + 2b/c + d` the terms are `a`, `2b/c` and `d`. To divide by `c + d`,
put it in parentheses; to divide the whole of `a + 2b`, put that in
parentheses too.

```lzr
a = 1, b = 2, c = 4, d = 8,
p = a + 2b/c + d,
q = a + 2b/(c + d),
r = (a + 2b)/(c + d),
PRINT p, q, r.
STOP
```
```page

p = +.10000000+02       q = +.13333333+01       r = +.41666665+00
```

These are the manual's (6), (7) and (8): 10, 4/3 and 5/12. "If it is merely
kept in mind that pluses and minuses act as separating symbols, no confusion
can arise."

## Within a term, what follows a slash is below the line

"As a further example, one can write an expression such as cd/a(b + c) + d.
The computer interprets this expression as the sum of the quantity d and the
ratio whose numerator is the product cd and whose denominator is the product
of a with the quantity b + c." (p. 4)

Everything after the slash, up to the end of the term, is in the denominator.
That is how a mathematician writes `x²/y (y - 1)` on one line, meaning x²
over y(y - 1), and it is how the manual's own cosine program is written
(chapter 14):

```lzr
a = 1, b = 2, c = 4, d = 8,
s = cd/a(b + c) + d,
x = 3,
y = 6,
z = x^2/y (y - 1),
PRINT s, z.
STOP
```
```page

s = +.13333333+02       z = +.30000001+00
```

`s` is 32/6 + 8, and `z` is 9/30.

## A second slash

Now the surprise.

"The use of more than one division within a single mathematical term is
admissible and is correctly interpreted by the computer. Thus a/b/c is
properly interpreted as the product ac divided by b." (pp. 4-5)

```lzr
a = 1, b = 2, c = 4,
u = a/b/c,
PRINT u.
STOP
```
```page

u = +.20000000+01
```

A slash does not divide by the next thing. It moves what follows to the other
side of the fraction line, and a second slash moves what follows it back.
`a/b/c` is a over b, then c back on top: ac/b, which is 2. In FORTRAN, and in
almost every language since, `a/b/c` is (a/b)/c, which is .125.

Neither reading is wrong; the manual says which one it means. But it is a
perfectly reasonable expression that a reader of today would get wrong. In
1976 Laning remembered writing the translator:

> The notion of operator precedence as a formal concept did not occur to me at
> the time; I lived in fear that someone would write a perfectly reasonable
> algebraic expression that my system would not analyze correctly.

## Parentheses start afresh

"An expression contained within parentheses is isolated from the rest of the
equation and the normal rules of multiplication and division apply without
regard to the remainder of the equation." (p. 5) So `a/(b + c/d)` is a over
b + c/d:

```lzr
a = 1, b = 2, c = 4, d = 8,
v = a/(b + c/d),
w = a/(b/c),
PRINT v, w.
STOP
```
```page

v = +.40000000+00       w = +.20000000+01
```

## How it is translated

`lz --translate` shows the block of orders an equation becomes. The factors on
top are multiplied, then those below, then there is one division:

```lzr --translate
u = a/b/c,
s = cd/a(b + c) + d,
STOP
```
```page
101  u = a/b/c,
       load  a
       load  c
       mul
       load  b
       div
       store u
102  s = cd/a(b + c) + d,
       load  c
       load  d
       mul
       load  a
       load  b
       load  c
       add
       mul
       div
       load  d
       add
       store s
103  STOP
       stop
```

## No more than four parentheses

"No more than four parentheses may be open in a single equation at any one
time" (p. 3). The manual's (4) is allowed and its (5) is not:

```lzr
a = 1, b = 1, c = 1, d = 1, e = 1, f = 1, g = 1, h = 1, i = 1, j = 1, k = 1,
x = a(b + c(d + e(f + g(h + i)(j + k)^2)^-1)),
PRINT x.
STOP
```
```page

x = +.21111111+01
```

```lzr
x = a(b + c(d + e(f + g(h + i(j + k)^2)^3))),
STOP
```
```page
<stdin>:1:30: error: no more than four parentheses may be open in a single equation at any one time
```
