# Sources

The PDFs are not in the repository. `sources/fetch.sh` downloads them here.

| File | Document | Where |
|---|---|---|
| `e364-laning-zierler-1954.pdf` | J. H. Laning Jr. and N. Zierler, *A Program for Translation of Mathematical Equations for Whirlwind I*, Engineering Memorandum E-364, MIT Instrumentation Laboratory, January 1954. 28 pages. **The specification this reconstruction follows.** | <https://web.mit.edu/digitalapollo/Documents/Chapter5/laningwhirlwind.pdf> |
| `summer-session-1954-advanced-coding.pdf` | *Digital Computers: Advanced Coding Techniques*, MIT Summer Session 1954, lecture 13, "The Algebraic Coding System of Laning and Zierler" (pp. 13-1 to 13-4), and the discussion after it. | <https://bitsavers.org/pdf/mit/whirlwind/summer_session_1954/Digital_Computers_Advanced_Coding_Techniques_Summer_1954.pdf> |
| `summary-report-37-1954q1.pdf` | Project Whirlwind Summary Report No. 37, first quarter 1954, item 108, "An Interpretive Program". | <https://bitsavers.org/pdf/mit/whirlwind/Project_Whirlwind_Summary_Report_No_37_First_Quarter_1954.pdf> |
| `knuth-trabb-pardo-1976-early-development.pdf` | D. E. Knuth and L. Trabb Pardo, *The Early Development of Programming Languages*, STAN-CS-76-562, August 1976, pp. 55-61. | <https://bitsavers.org/pdf/stanford/cs_techReports/STAN-CS-76-562_EarlyDevelPgmgLang_Aug76.pdf> |
| `laning-to-knuth-1976-07-02.pdf` | J. H. Laning Jr., letter to D. E. Knuth, 2 July 1976: 17 pages, the 11-page letter and some of its enclosures, among them the 1953 listing (enclosure 1, not reproduced) and the lead-pursuit program with its first output page (enclosure 2, pp. 12-15). **The only output of the 1954 system we have found.** | <https://cs.stanford.edu/~knuth/from-Laning-76-07-02.pdf> |
| `knuth-to-laning-1976-08-16.pdf` | D. E. Knuth, letter to Laning, 16 August 1976: the 1953 listing "has only one bug ... unary minus". | <https://cs.stanford.edu/~knuth/to-Laning-76-08-16.pdf> |
| `laning-1953-listing.pdf` | J. H. Laning Jr., the translator's listing, 3/53, "Tape T-2134-m8 plus correction tape P-2134-10", 20 sheets. Knuth archive U2774, Computer History Museum. | <https://archive.computerhistory.org/resources/text/Knuth_Don_X4100/PDF_index/k-3-pdf/k-3-u2774-Whirlwind-program.pdf> |
| `knuth-1976-notes-on-1953-listing.pdf` | D. E. Knuth, "Notes on Laning's 1953 Whirlwind program", 12-14 August 1976, 4 pages. | <https://archive.computerhistory.org/resources/text/Knuth_Don_X4100/PDF_index/k-3-pdf/k-3-u2774-Whirlwind-notes.pdf> |

`*.txt` beside them are `pdftotext -layout` of the two with a text layer. The
OCR in them is poor; every passage quoted in this repository was read from the
page images (see [`../docs/scans/`](../docs/scans/README.md)).

## What each adds

**E-364** is a programmer's manual, not a listing: it says what the system does,
with examples, and not how. Everything here that is not in it is listed under
"What is not original" in the README.

**Summary Report 37** (item 108): from the first quarter of 1954 the problem
tape went into the photoelectric reader and was stored on the drum; the system
was on one master tape, 108-60-30; "All the routines described in E-364 have
been tested and work correctly with the exception of the X|N routine"; and a
report "on the inner workings ... has been started by N. Zierler. However, it
will be some time before this report reaches completion, since the author
transferred from the Instrumentation Laboratory to Project Lincoln." We have
not found such a report, though no thorough archival search has been made.

**The summer session notes** describe the system again, give `a|N = 2(2)8,`,
repeat the D-equation program without its PRINT (see
`examples/1954-summer-session.lzr`), and report: "One problem involved a set of
six simultaneous differential equations ... Coding required only a few hours and
the routine ran successfully the first time. Computing time was about six to
eight times as long as it would have been using the single-address
interpretive system ordinarily used at the Digital Computer Laboratory", and
"with a single exception, all programs coded in the Laning-Zierler system have
been completed successfully on their first run." The discussion records that
"there are no annotated copies of the conversion routine", that indices "are
rounded off to the nearest integer", and J. W. Backus asking about it.

**Knuth and Trabb Pardo** write TPK in the system (`examples/1976-knuth-tpk.lzr`),
quote C. W. Adams's talk of May 1954 (credited to Adams and Laning, though
Laning "did not write the paper or attend the conference", p. 97) on "a reduction of computing speed of the order of
magnitude of ten to one from an efficient computer program", and quote Laning in
1976: the lead-pursuit program was written "in a 2-1/2 hour coding session" and
"ran successfully the first time"; it "consisted of 79 statements, including 29
which merely assigned initial data values, and also including seven uses of the
differential equation feature". And: "I lived in fear that someone would write a
perfectly reasonable algebraic expression that my system would not analyze
correctly."

**Laning's letter of 1976** sends Knuth "what I believe to be the prototype
Whirlwind algebraic compiler, complete with coffee stains" and "the first
effort to use the Whirlwind compiler on a serious engineering problem", the
lead-pursuit program, which Laning guessed was from "very early 1954". Its
program and first output page are `examples/1954-lead-pursuit.lzr` and
`examples/1954-lead-pursuit.whirlwind`. Of unary minus he wrote: "one
almost-debugged version which was believed correct for a while would not
handle the minus sign properly in 'X = -1'. I mention this partly because I am
not sure whether the version I am sending you does or does not contain this
error." Knuth found that it did.

**The 1953 listing and Knuth's notes** are an earlier and smaller system than
E-364's, without superscript variables, the drum, `D` equations, `SR`, `CR` or
functions ("not yet implemented", Knuth's notes p. 4). It has been transcribed
and run on a simulated Whirlwind by L. A. Stabile, on G. Fedorkow's simulator:
<https://github.com/gfedorkow/Whirlwind-Instruction-Simulator>, under
`Code-Samples/Laning-and-Zierler-Interpreter`, which also holds Knuth's 1976
correspondence with Laning and Stabile's paper on the work (March 2026).
