# 15. What became of it

## Backus came to see it

In May 1954 the Office of Naval Research held a symposium on automatic
programming, and Charles Adams of MIT described the Laning and Zierler system
there. John Backus, at IBM, was already thinking along the same lines. Knuth
and Trabb Pardo tell what happened next (KTP 76, p. 60):

> Shortly after learning of the Laning and Zierler system at the ONR meeting
> in May, Backus wrote to Laning that "our formulation of the problem is very
> similar to yours: however, we have done no programming or even detailed
> planning." Within two weeks, Backus and his co-workers Harlan Herrick and
> Irving Ziller visited M.I.T. in order to see the Laning/Zierler system in
> operation. The big problem facing them was to implement such a language with
> suitable efficiency.

That summer, at MIT's summer session on advanced coding, the system was
described again, and Backus and Grace Hopper were both in the discussion that
followed. The notes record Backus on two points: he "emphasised that the
rounding off of indices is done during execution, not during the reading of
the program", and "stated that equations are stored individually on the drum"
(SS 54, p. 13-5). The first is the rounding of
[chapter 9](09-superscript-variables.md). Asked why the functions were F¹,
F², ... and not sin and cos, "Prof. Adams said that Laning and Zierler were
tired of adding ingenuities when they reached this point."

Grace Hopper, two years later, put it plainly (KTP 76, p. 55):

> A description of Laning and Zierler's system of algebraic pseudocoding for
> the Whirlwind computer led to the development of Boeing's BACAIC for the
> 701, FORTRAN for the 704, AT-3 for the Univac, and the Purdue System for the
> Datatron and indicated the need for far more effort in the area of algebraic
> translators.

## How slow it was

Adams's talk at that symposium gave the cost (KTP 76, p. 58):

> The use of a small number of standard closed subroutines has certain
> advantages of logical simplicity; however, it also often results in the
> execution of numerous unnecessary operations. This fact, plus the frequent
> reference to the drum required in calling in equations, results in a
> reduction of computing speed of the order of magnitude of ten to one from an
> efficient computer program.

The paper names Laning as co-author, but Knuth and Trabb Pardo record that he
"did not write the paper or attend the conference" (KTP 76, p. 97).

The summer session put it against something nearer: "Computing time was about
six to eight times as long as it would have been using the single-address
interpretive system ordinarily used at the Digital Computer Laboratory" (SS
54, p. 13-4). Both are measures against a program written for Whirlwind by
hand, and neither says how the time was measured.

`lz` does not keep time. `lz --stats` counts the work: each equation brought
from the drum, each floating-point operation. The only time it gives is the
manual's thirty milliseconds for a variable on the drum (p. 16):

```lzr --stats
x = 0,
1   z = 1 - x^2/2 + x^4/2·3·4 - x^6/2·3·4·5·6
        + x^8/2·3·4·5·6·7·8 - x^10/ 2·3·4·5·6·7·8·9·10,
    x = x + .1,
    e = x - 1.05,
    CP 1,
    STOP
```
```page
equations carried out      46
blocks brought from drum   34
interpretive orders        793
floating-point operations  682
function subroutine calls  0
drum variable accesses     0  (about 30 ms each, E-364 p. 16: 0.0 s)
integration steps          0
values printed             0
```

That is the manual's table of cosines without its PRINT. Every multiplication
and division in it is a call of an interpretive routine, and every equation is
brought from the drum before it is done.

Laning, eleven years later, did not argue with the figures (KTP 76, p. 58):

> This was in the days when machine time was king, and people-time was
> worthless (particularly since I was not even on the Whirlwind staff). ...
> [The program] did perhaps pay for itself a few times when a complex problem
> required solutions with a twenty-four hour deadline.

## What was never finished

The manual's two temporary restrictions ([chapter 13](13-temporary-rules.md))
were "expected" to "be removed in the immediate future" (p. 21). Three months
later one routine still did not work: "All the routines described in E-364 have
been tested and work correctly with the exception of the X|N routine." The same
report says that a report "on the inner workings and programming methods used
in this interpretive program has been started by N. Zierler. However, it will
be some time before this report reaches completion, since the author
transferred from the Instrumentation Laboratory to Project Lincoln." (SR 37,
p. 17) No such report has been found. At the summer session Donn Combelic said
the system "had taken about a year and a half to develop, and had not been
finished thoroughly. There are no annotated copies of the conversion routine,
and when the next algebraic coding scheme is developed it will be done from
scratch." (SS 54, p. 13-5) That is why everything in this repository was
rebuilt from the manual, not from the program.

## A slip in the manual

The manual's figure for the largest number is wrong
([chapter 4](04-numbers.md)). Page 6 gives 2⁶³ - 2³⁹ as "roughly,
9.2233995 · 10¹⁸"; it is 9.2233715 · 10¹⁸. Nothing on the page depends on it,
and the example that follows it, "9223314040000000000 is a permissible
number", is right either way. It is the kind of thing the authors asked to be
told about.

## MAC

The system was not developed further. "Plans for a much expanded Whirlwind
compiler were dropped when the M.I.T. Instrumentation Lab acquired its own
computer, an IBM 650. Laning and his colleagues Philip C. Hankins and Charles
P. Werner developed a compiler called MAC for this machine in 1957 and 1958."
MAC wrote a statement on three lines, with exponents on the line above and
subscripts on the line below. It "was very
successful locally: MAC compilers were later developed for use with IBM 704,
709, 7090 and 360 computers, as well as the Honeywell H800 and H1800 and the
CDC 3600", and in 1976 "MAC and FORTRAN have about equal use" at the Draper
Laboratory, the Instrumentation Laboratory's successor (KTP 76, p. 60).

## What Knuth and Trabb Pardo thought of it

"The full user's manual ought to be reprinted some day because their language
went so far beyond what had been implemented before. The programmer no longer
needed to know much about the computer at all, and the user's manual was (for
the first time) addressed to a complete novice." (KTP 76, p. 56) And: "except
for control statements -- for which there was no existing mathematical
convention -- Laning and Zierler's notation was quite easy to read" (p. 57).

Next: [Reference](16-reference.md)
