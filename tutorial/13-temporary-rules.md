# 13. The temporary rules of 1954

## What the manual admitted

The last section of E-364 is half a page, and unusual in a manual:

> 18. Temporary Rules and Restrictions
>
> As of the date at which this report is written the following restrictions
> are in effect. It is expected that they will be removed in the immediate
> future.
>
> 1. An SR or CR instruction may not refer to an equation that makes use of a
>    subroutine.
> 2. The x|N is not yet in operation.
>
> All other portions of the program are believed to be operating
> satisfactorily but exhaustive tests have not been made. Information relative
> to any troubles encountered in the use of this program would be appreciated
> by the authors. (p. 21)

## What happened next

Project Whirlwind's report for the first quarter of 1954 (Summary Report 37,
item 108) says: "All the routines described in E-364 have been tested and work
correctly with the exception of the X|N routine." (SR 37, p. 17) That suggests
that by the end of March the first restriction had gone and the second had
not, though the report does not mention the restriction itself. Nothing later has been found
that says whether x|N was ever put right.

## The rules here

By default `lz` is the system as the manual describes it, everything working.
`lz --temporary-rules` is the system of January 1954, with both restrictions:

```lzr
y = 16,
1 x = F^1(y),
SR 1,
PRINT x.
STOP
```
```page

x = +.40000000+01
```

```lzr --temporary-rules
y = 16,
1 x = F^1(y),
SR 1,
PRINT x.
STOP
```
```page
<stdin>:3:1: error: "An SR or CR instruction may not refer to an equation that makes use of a subroutine" (E-364, section 18): equation 1 calls a function
```

```lzr --temporary-rules
g|N = 2, 4, 6, 0,
PRINT g|3.
STOP
```
```page
<stdin>:1:1: error: "The x|N is not yet in operation" (E-364, section 18, as of January 1954)
```

What failed in 1954 is not known, only that it did; here the restriction is
enforced by refusing to translate the program, and quoting the manual.

## Why it matters

x|N is the short way to put a table of numbers on the drum; without it, each
value is an equation of its own, and "100 unnumbered equations, the maximum"
(p. 17) soon run out. Knuth and Trabb Pardo's TPK program
([chapter 14](14-worked-examples.md)) begins with one. Under the rules of
January 1954 it could not have run.

Next: [Worked examples](14-worked-examples.md)
