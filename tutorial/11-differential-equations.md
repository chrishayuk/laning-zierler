# 11. Differential equations

## D

"Provision has been made for the solution of ordinary differential equations
by the method of Gill - a variation of the 4th order Runge-Kutta technique.
In order to make use of this provision, certain conventions must be
observed." (p. 17)

1. the letter `t` must be used for the independent variable;
2. the letter `h` must be used for the increment in t;
3. the upper case letter `D` is used to denote d/dt.

"A given nth order system must be reduced to a system of n first order
differential equations." (p. 18) Then each is written with a D in front of the
variable it is the derivative of. `Dx = y,` means dx/dt = y.

The simplest: dy/dt = y, from y = 1, which is eᵗ.

```lzr
t = 0,
h = .1,
y = 1,
1 Dy = y,
e = t - .95,
CP 1,
PRINT t, y.
STOP
```
```page

t = +.10000001+01       y = +.27182793+01
```

y is 2.7182793, where e itself is 2.7182818: Gill's method with a step of .1
is that close. "It should be observed that t is automatically increased by
the value h upon completion of the last equation that starts with a D." (p.
19) The
loop round equation 1 takes ten steps, and t is 1.0000001 because a tenth is
not exact in binary.

## The manual's example

Two equations, dy₁/dt = y₂ + 1 and dy₂/dt = -y₁, from t = 2 with both zero,
h = .5, printed at t = 2, 3, ..., 10 (p. 18):

```lzr
    t = 2,
    h = 0.5,
    y|1 = 0,
    y|2 = 0,
1   PRINT t, y|1, y|2.
    e = 9.7 - t,
    CP 3,
2   D y|1 = y|2 + 1,
    D y|2 = - y|1,
    e = -e,
    CP 2,
    SP 1,
3   STOP
```
```page

t = +.20000000+01       y|1 = +.00000000+00     y|2 = +.00000000+00

t = +.30000000+01       y|1 = +.84103733+00     y|2 = -.45941162+00

t = +.40000000+01       y|1 = +.90930998+00     y|2 = -.14151079+01

t = +.50000000+01       y|1 = +.14244115+00     y|2 = -.19891661+01

t = +.60000000+01       y|1 = -.75492370+00     y|2 = -.16545300+01

t = +.70000000+01       y|1 = -.95858722+00     y|2 = -.71891236+00

t = +.80000000+01       y|1 = -.28179597+00     y|2 = -.41839677-01

t = +.90000000+01       y|1 = +.65351301+00     y|2 = -.24502874+00

t = +.10000000+02       y|1 = +.98824048+00     y|2 = -.11414999+01
```

The answer is y₁ = sin(t - 2) and y₂ = cos(t - 2) - 1; at t = 3 that is .841471
and -.459698. The step of .5 is large, and the fourth digit already differs.

Notice `e = -e, CP 2`. e is 9.7 - t from before the step, positive. Negated,
it is negative, so CP 2 takes a second step; negated again it is positive, so
the program goes on to SP 1 and prints. Two steps for each line.

## Four passes

"The carrying forward of the solution a single step of length h actually
involves going through the part of the program that starts with the first D
equation and ends with the last D equation a total of 4 times. These 4
sub-steps take place automatically" (p. 19).

`lz --trace` shows them. Each D equation is computed four times. The manual
says only that t has gone up by h at the end of the fourth pass. Here, as in
the classical fourth-order Runge-Kutta method and Gill's variation of it, it is
t + h/2 for the second and third passes and t + h for the fourth; that is the
reconstruction's reading, not the manual's words. The trace shows the derivative f at each pass (what is kept is h·f),
and `lz` writes the trace after the page:

```lzr --trace
t = 0,
h = .5,
x = 1,
v = 0,
Dx = v,
Dv = -x,
PRINT t, x, v.
STOP
```
```page

t = +.50000000+00       x = +.87760418+00       v = -.47916665+00
101  t = +.00000000+00
102  h = +.50000000+00
103  x = +.10000000+01
104  v = +.00000000+00
105  Dx = +.00000000+00   (pass 1)
106  Dv = -.10000000+01   (pass 1)
105  Dx = -.25000000+00   (pass 2)
106  Dv = -.10000000+01   (pass 2)
105  Dx = -.25000000+00   (pass 3)
106  Dv = -.96338832+00   (pass 3)
105  Dx = -.46875000+00   (pass 4)
106  Dv = -.87500000+00   (pass 4)
     t = +.50000000+00
```

After each of the first three passes, control goes back to the first D
equation; after the fourth, it goes on to whatever follows the last one.

## The change that would appear to be innocuous

Because of those four passes, "it is essential that all relevant auxiliary
computation take place between the first and last D equations." The manual's
example changes its program so that a value used by a D equation is computed
just before the first one:

```lzr
    t = 2,
    h = 0.5,
    y|1 = 0,
    y|2 = 0,
1   PRINT t, y|1, y|2.
    e = 9.7 - t,
    CP 3,
2   a = y|2 + 1,
    D y|1 = a,
    D y|2 = - y|1,
    e = -e,
    CP 2,
    SP 1,
3   STOP
```
```page

t = +.20000000+01       y|1 = +.00000000+00     y|2 = +.00000000+00

t = +.30000000+01       y|1 = +.93750000+00     y|2 = -.48437500+00

t = +.40000000+01       y|1 = +.11865234+01     y|2 = -.16130371+01

t = +.50000000+01       y|1 = +.31517028+00     y|2 = -.24284629+01

t = +.60000000+01       y|1 = -.11028064+01     y|2 = -.20320234+01

t = +.70000000+01       y|1 = -.17946268+01     y|2 = -.49825602+00

t = +.80000000+01       y|1 = -.87558519+00     y|2 = +.94117444+00

t = +.90000000+01       y|1 = +.11631622+01     y|2 = +.82177931+00

t = +.10000000+02       y|1 = +.25802898+01     y|2 = -.11511096+01
```

"This change would appear to be innocuous since 'a' seems to be appropriately
reset before each entry into the differential equations. However, upon
completion of the second D equation in each of the first 3 sub-steps,
computation would be resumed with the first D equation (and not equation 2) so
that 'a' would not have the current value of y|2 + 1." (p. 19) The first line,
at t = 2, is the same; from the first step on the answers are wrong: .9375 at
t = 3 instead of .841.

`lz --check` knows the rule:

```lzr --check
    t = 2,
    h = 0.5,
    y|1 = 0,
    y|2 = 0,
1   PRINT t, y|1, y|2.
    e = 9.7 - t,
    CP 3,
2   a = y|2 + 1,
    D y|1 = a,
    D y|2 = - y|1,
    e = -e,
    CP 2,
    SP 1,
3   STOP
```
```page
<stdin>:8:1: warning: a is computed from the solution outside the D equations but used inside them: each step goes through the D equations four times, and a is not recomputed between passes. "It is essential that all relevant auxiliary computation take place between the first and last D equations" (p. 19)
```

An auxiliary value that the first D equation itself needs cannot go between
the first and the last. Write it into the D equation instead, as
[`examples/pursuit.lzr`](../examples/pursuit.lzr) does.

## Storage

"It has been stated that a total of 250 MS superscript variables may be used;
this is not true for programs involving differential equations. If there are
k equations that begin with a D in a program then 250 - 2k MS superscript
variables may be used." (p. 19) So each D equation took the room of two
superscript variables.
The manual does not say what they held. Here they are the derivative times h
and Gill's running correction, q, which is what makes Gill's variation careful
with rounding; that fits "250 - 2k", and is the reconstruction's choice.

## How Gill's method goes

For each pass j = 1, 2, 3, 4 and each D equation, with k = h·f the derivative
times the step:

| pass | r = | then y = y + r and q = |
|---|---|---|
| 1 | ½ (k - 2q) | q + 3r - ½k |
| 2 | (1 - √½)(k - q) | q + 3r - (1 - √½)k |
| 3 | (1 + √½)(k - q) | q + 3r - (1 + √½)k |
| 4 | ⅙ (k - 2q) | q + 3r - ½k |

The manual names the method and no more. These are the standard formulas, done
in 24-bit arithmetic; all the derivatives of a pass are worked out before any
variable moves.

Next: [Post-mortems and putting a tape right](12-post-mortems.md)
