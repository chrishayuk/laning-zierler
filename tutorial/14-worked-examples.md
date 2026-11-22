# 14. Worked examples

## A table of cosines

The manual's first full program (p. 11): "Suppose it is desired to print a
table of values of cosine x for x = 0, .1, .2, ..., 1 radian and that we
decide to use up to the x¹⁰ term in the power series for cosine x."

```lzr
x = 0,
1   z = 1 - x^2/2 + x^4/2·3·4 - x^6/2·3·4·5·6
        + x^8/2·3·4·5·6·7·8 - x^10/ 2·3·4·5·6·7·8·9·10,
    PRINT x, z.
    x = x + .1,
    e = x - 1.05,
    CP 1,
    STOP
```
```page

x = +.00000000+00       z = +.10000000+01

x = +.10000000+00       z = +.99500417+00

x = +.20000000+00       z = +.98006659+00

x = +.30000001+00       z = +.95533645+00

x = +.40000000+00       z = +.92106103+00

x = +.50000000+00       z = +.87758260+00

x = +.60000002+00       z = +.82533562+00

x = +.70000004+00       z = +.76484221+00

x = +.80000007+00       z = +.69670671+00

x = +.90000009+00       z = +.62160986+00

x = +.10000001+01       z = +.54030215+00
```

Every term is written the way it would be on paper: `x^4/2·3·4` is x⁴ over
2·3·4, because what follows a slash is below the line until the next sign
([chapter 3](03-the-slash.md)). And the loop stops at 1.05, not 1, so that
the test is never zero ([chapter 8](08-zero.md)). cos 1 is .54030231, and the
last line is .54030215. Stopping the series at x¹⁰ costs only about 2·10⁻⁹.
Most of the difference is x: after ten additions of .1 it is 1.0000001
([chapter 4](04-numbers.md)), and the cosine of that is .54030221. The 24-bit
arithmetic of six terms loses the rest. The second version, below, reaches x =
1 exactly, and gives .54030233.

The second version (p. 11) works out the series from the inside, "the
advantage that the number of terms in the series may be adjusted by changing a
single number". On the page `y = y - 2` ends in what looks like a semicolon (or
a comma with a speck above it); it is typed with a comma here. Its test,
`e = 1 - y,` with y even, cannot be zero:

```lzr
    x = 0,
1   y = 10,
    z = 1,
2   z = 1 - z x^2/y (y - 1),
    y = y - 2,
    e = 1 - y,
    CP 2,
    PRINT x, z.
    x = x + 0.5,
    a = x - 1.05,
    CP 1,
    STOP
```
```page

x = +.00000000+00       z = +.10000000+01

x = +.50000000+00       z = +.87758255+00

x = +.10000000+01       z = +.54030233+00
```

(Here the step is .5 instead of .1, to keep the page short.) `z x^2/y (y - 1)`
is z x² over y(y - 1): the whole of `y (y - 1)` is below the line.

## TPK

In 1976 Donald Knuth and Luis Trabb Pardo wrote the same small program, TPK, in
each of the early languages, to compare them. This is how they wrote it for
Laning and Zierler's system (KTP 76, pp. 56-57). They wrote it to show "how
TPK would look in their system"; as far as is known it was never run on
Whirlwind. It is incompatible with the 1954 specification in two places, and
what follows is what the manual's rules, and this reconstruction, make of it.

```text
 1          v|N =    ⟨input⟩,
 2          i = 0,
 3     1    j = i+1,
 4          a|i = v|j,
 5          i = j,
 6          e = i-10.5,
 7          CP 1,
 8          i = 10,
 9     2    y = F¹(F¹¹(a|i))+5(a|i)³,
10          e = y-400,
11          CP 3,
12          z = 999,
13          PRINT i,z.
14          SP 4,
15     3    PRINT i,y.
16     4    i = i-1,
17          e = -0.5-i,
18          CP 2,
19          STOP
```

It reads eleven numbers, then goes through them backwards and prints
f(t) = √|t| + 5t³ for each, or 999 if that is more than 400. The numbers on
the left are Knuth and Trabb Pardo's line numbers, not part of the program.
For the input they suggest the abbreviation `v|N = 1 (.5) 2 (.25) 3.5 (1) 5.5`
(KTP 76, p. 57), and that is what the runs below use in place of ⟨input⟩.

Lines 2 to 7 move the eleven numbers from the drum to a|0 ... a|10 in core.
Line 6 subtracts 10.5, and line 17 subtracts from -0.5: they kept the
manual's habit of never testing a zero. `lz --check` has nothing to say about
those, but it does about line 16:

```lzr --check
        v|N = 1 (.5) 2 (.25) 3.5 (1) 5.5,
        i = 0,
1       j = i + 1,
        a|i = v|j,
        i = j,
        e = i - 10.5,
        CP 1,
        i = 10,
2       y = F^1(F^11(a|i)) + 5(a|i)^3,
        e = y - 400,
        CP 3,
        z = 999,
        PRINT i, z.
        SP 4,
3       PRINT i, y.
4       i = i - 1,
        e = -0.5 - i,
        CP 2,
        STOP
```
```page
<stdin>:11:9: hint: if e comes out exactly zero, "the CP may or may not behave like an SP since 0 may appear as +0 or -0" (p. 8). The manual's own loops compare with a half: n - 399.5, not n - 400
<stdin>:16:1: hint: i is used as an index and counted down by arithmetic. If it reaches zero this way it may not be used as one: zero is permissible "if assigned by the specific equation i = 0," (p. 13)
```

(Its other hint, on `CP 3`, is about y being exactly 400, which it never is.)
Here is the run:

```lzr
        v|N = 1 (.5) 2 (.25) 3.5 (1) 5.5,
        i = 0,
1       j = i + 1,
        a|i = v|j,
        i = j,
        e = i - 10.5,
        CP 1,
        i = 10,
2       y = F^1(F^11(a|i)) + 5(a|i)^3,
        e = y - 400,
        CP 3,
        z = 999,
        PRINT i, z.
        SP 4,
3       PRINT i, y.
4       i = i - 1,
        e = -0.5 - i,
        CP 2,
        STOP
```
```page

i = +.10000000+02       z = +.99900000+03

i = +.90000000+01       z = +.99900000+03

i = +.80000000+01       y = +.21624583+03

i = +.70000000+01       y = +.17344340+03

i = +.60000000+01       y = +.13673205+03

i = +.50000000+01       y = +.10564268+03

i = +.40000000+01       y = +.79706138+02

i = +.30000000+01       y = +.58453125+02

i = +.20000000+01       y = +.41414215+02

i = +.10000000+01       y = +.18099744+02

ALARM  EQUATION 2  PRECEDED BY 109
lz: alarm: i is -0, a zero computed by arithmetic: as a superscript zero is permissible only if assigned by the specific equation i = 0,
```

Ten lines, and then an alarm in equation 2 on the last number. i has come down
to zero by `i = i - 1`, and E-364 permits a zero index only "if assigned by the
specific equation n = 0," (p. 13; [chapter 9](09-superscript-variables.md)).
That rule is the incompatibility, and it does not depend on the sign: the
program uses an index the 1954 specification does not permit. The alarm is
another matter. It rests on the reconstruction's model of zero, in which such
a zero is -0, which is negative ([chapter 8](08-zero.md)). The manual does not
say what Whirlwind did with the zero, and this run does not show that it
failed.

With every zero +0 it finishes. That is a what-if, not the manual: the rule
forbids the computed zero whatever its sign, and `--zero plus` lets it through
only because the zero it computes is +0, as one set by `i = 0,` is.

```lzr --zero plus
        v|N = 1 (.5) 2 (.25) 3.5 (1) 5.5,
        i = 0,
1       j = i + 1,
        a|i = v|j,
        i = j,
        e = i - 10.5,
        CP 1,
        i = 10,
2       y = F^1(F^11(a|i)) + 5(a|i)^3,
        e = y - 400,
        CP 3,
        z = 999,
        PRINT i, z.
        SP 4,
3       PRINT i, y.
4       i = i - 1,
        e = -0.5 - i,
        CP 2,
        STOP
```
```page

i = +.10000000+02       z = +.99900000+03

i = +.90000000+01       z = +.99900000+03

i = +.80000000+01       y = +.21624583+03

i = +.70000000+01       y = +.17344340+03

i = +.60000000+01       y = +.13673205+03

i = +.50000000+01       y = +.10564268+03

i = +.40000000+01       y = +.79706138+02

i = +.30000000+01       y = +.58453125+02

i = +.20000000+01       y = +.41414215+02

i = +.10000000+01       y = +.18099744+02

i = +.00000000+00       y = +.60000000+01
```

The other incompatibility is its first line, an x|N, which "is not yet in
operation" under the rules of January 1954 ([chapter 13](13-temporary-rules.md)).
`lz --temporary-rules` refuses to translate it.

## A pursuit curve

Laning's first serious use of the system, as he remembered it in 1976, was "a
three-dimensional lead pursuit course flown by one aircraft attacking another,
including the fire control equations." He and Zierler sat down with the
colleagues who had formulated it "in a 2-1/2 hour coding session, at least half
of which was spent in defining notation. The tape was punched, and with the
usual beginner's luck it ran successfully the first time!" (KTP 76, p. 59)
Laning still had the listing, and Knuth and Trabb Pardo give its size: "79
statements, including 29 which merely assigned initial data values, and also
including seven uses of the differential equation feature."

That listing is not public. Here is the flat, simple version. A target flies
up the line x = 1 at speed 1, starting at (1, 0). A pursuer starts at the
origin, flies at speed w = 1.25, and always points at the target. With d the
distance between them:

x' = w(1 - x)/d,   y' = w(t - y)/d

Both D equations need d. Auxiliary computation has to be between the first
and the last D equation ([chapter 11](11-differential-equations.md)), and
nothing can be between them and the first, so d is written out in both:

```lzr
    t = 0,
    h = 0.05,
    w = 1.25,
    x = 0,
    y = 0,
    n = 0,
1   Dx = w(1 - x)/F^1((1 - x)^2 + (t - y)^2),
    Dy = w(t - y)/F^1((1 - x)^2 + (t - y)^2),
    d = F^1((1 - x)^2 + (t - y)^2),
    n = n + 1,
    e = n - 7.5,
    CP 2,
    n = 0,
    PRINT t, x, y, d.
2   e = d - .05,
    CP 3,
    SP 1,
3   PRINT t, x, y, d.
    STOP
```
```page

t = +.40000003+00       x = +.48117771+00       y = +.11495894+00       d = +.59196710+00

t = +.80000013+00       x = +.82428610+00       y = +.46873179+00       d = +.37498545+00

t = +.11999999+01       x = +.96520841+00       y = +.94576078+00       d = +.25660863+00

t = +.15999995+01       x = +.99708455+00       y = +.14444595+01       d = +.15556731+00

t = +.19999991+01       x = +.99998247+00       y = +.19444444+01       d = +.55554747-01

t = +.20499992+01       x = +.99999469+00       y = +.20069444+01       d = +.43054819-01
```

The theory says the pursuer catches the target at t = w/(w² - 1) = 2.22. The
program stops when they are less than .05 apart; closing at w - 1 = .25 a
second from behind, that is about .2 before the end. [`examples/pursuit.lzr`](../examples/pursuit.lzr) has
a smaller step and goes closer.

## More

[`examples/`](../examples/) has the rest: the manual's interpolation in a
table, Newton's method, primes found with a rounded index, Simpson's rule, a
pendulum, a shell with air resistance, predators and prey, the Lorenz
equations, and others. Each begins with a note saying what it does and what the
answer should be.

Next: [What became of it](15-what-became-of-it.md)
