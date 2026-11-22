# Independent review, November 2026

## Who reviewed it

OpenAI's Codex CLI (codex-cli 0.166.0), on 22 November 2026: a second
language model, independent of the one that wrote this repository, not a
person. It was asked to check the repository's code, its claims and its
transcriptions against the original documents in `sources/`. It changed no
tracked file.

Its findings were then checked one by one by the model that wrote the
repository (Claude), against the code, by running `lz`, and against the page
images, before anything was changed. No person has yet checked either.

## What was checked

- **Transcriptions.** Codex compared 20 places in the repository with the page
  images (operators, signs, digits, indices, labels, punctuation): 17 matched,
  or differed only in ways already disclosed (superscripts typed as `^`,
  `PRINT` and `STOP` added, values supplied); 3 had changes or discrepancies
  that were not disclosed (E-364 pp. 7, 8 and 11, below). It also found that
  two scan files were named after the wrong pages.
- **Re-check.** Those 3 places, and E-364 pp. 5, 13, 18 and 19 and Summer
  Session pp. 13-3 and 13-4 (9 pages in all), were looked at again from the
  page images. All of Codex's readings were confirmed.
- **Code.** Codex ran the build, the tests, the playground's tests and the
  extension's, and tried small programs against the claims in the README.
- **Claims.** Codex read the README, the tutorial and the tool text for claims
  the sources do not support, and checked the historical claims against Knuth
  and Trabb Pardo, Summary Report 37 and the Summer Session notes.
- **Numbers.** Codex recomputed 2⁶³ - 2³⁹, the chained exponent examples and,
  independently of `lz`, the fourth-order solution of the section 15
  differential equations: all nine printed rows agree with it to about
  3 · 10⁻⁷.

## The findings, and what was done

Each finding is numbered as in the review.

1. **The interpreter does not enforce the zero-index rule; it uses the sign of
   zero as a proxy.** Accepted in part. Confirmed: `n = 0·5,` and
   `n = 1, n = n - 1, n = F^11(n),` both give a +0 index, and both are let
   through although p. 13 forbids a computed zero. The code was not changed:
   the sign is a model of how a machine might have behaved, not a rule-checker,
   and a check on how a value was assigned would be an alarm nobody has
   evidence for. The README, the tutorial (chapters 9, 12 and 16), the hover
   text and the source comments now say that the rule is approximated, not
   enforced, and give the cases it lets through. A test pins the behaviour
   (`machine::tests::the_index_rule_is_a_proxy`, commit 675c33b).
2. **`--zero plus` does not make every zero +0.** Accepted; a bug. Negation
   flipped the sign of a zero without applying the zero model, so `n = -n`
   with n = +0 gave -0 under `--zero plus`: `CP` jumped on it and an index
   alarmed. A `-0` in a list on the drum, and a sum that underflowed, had the
   same fault. Fixed in commit 675c33b, with tests
   (`machine::tests::plus_zero_covers_negation_and_lists`,
   `num::tests::plus_zero_signs_every_zero`). No example or tutorial page
   changed.
3. **"Printing truncates" is stronger than the evidence.** Accepted. The
   manual's printed forms are illustrations, not a transcript, and the
   argument depends on the reconstruction's own rounding of .05 on input. The
   README, tutorial chapter 4 and the comment in `src/num.rs` now call
   truncation a choice that fits the illustrations (commits 8d4bb68, 9b9c2e5).
4. **Two punctuation repairs made silently, and a discrepancy in a figure not
   reported.** Accepted. Confirmed from the page images: p. 8 prints
   `x = 3 ab` without its comma (the prose below writes it with one); p. 11
   prints `y = y - 2;` (a semicolon, or a comma with a speck above it); the
   figure on p. 7 shows a ninth value, `i`, for `PRINT a, ..., h.` Each is now
   disclosed in the example's header, in the tutorial and in the README's new
   table of corrections, labelled a slip (commits 505ddf0, 9b9c2e5). The
   programs themselves are unchanged.
5. **The exponent examples go out of range, but the particular alarm is the
   reconstruction's.** Accepted. Taking the reciprocal first would overflow
   instead of dividing by zero. The README and the example now say so
   (commits 505ddf0, 9b9c2e5).
6. **The summer-session listing: the missing commas are slips, the added
   `PRINT` is an interpretation, and as printed the listing does not even
   translate.** Accepted. Confirmed by running the listing as printed. The
   README and the example no longer say it "types nothing" as a defect: a
   routine may leave its results in storage. The `PRINT` is labelled ours
   (commits 505ddf0, 9b9c2e5).
7. **Smaller claims to narrow.** Accepted, each:
   - "the manual says so twice" (signed zero): p. 13 does not mention the
     sign. Now "once", with p. 13 described separately.
   - "every loop" in the manual uses the half: the iterative cosine avoids
     zero by parity. Corrected in tutorial chapter 8 and the README.
   - "two words of storage" per D equation: the manual counts superscript
     variables, not machine words. Corrected in tutorial chapter 11.
   - "as in any Runge-Kutta method": now "the classical fourth-order method and
     Gill's variation of it".
   - "FORTRAN and every language since": now "most later languages".
   - "the Flexowriter had no such character" (`#`): the sources do not say.
     Now "E-364 describes no way of writing a comment" (README, tutorial
     chapter 1, hover text).
   - "What was never finished", "No such report has been found", and "no
     annotated copies" read as "no copy": narrowed to what we have not found,
     with no thorough archival search made (tutorial chapter 15, README,
     `sources/README.md`).
   - TPK "was never run on Whirlwind": now "as far as is known".
   - `docs/scans/summer-1954-13-2.jpg` and `-13-3.jpg` show pp. 13-3 and 13-4:
     renamed (commit b1468b7).
8. **TPK: the incompatibility is supported; a historical failure is not.**
   Accepted. Codex noted that the repository already disclosed the alarm's
   dependence on the zero model. The framing is now the same everywhere
   (README, tutorial chapters 9, 13 and 14, the example's header): TPK, an
   example written in 1976, is incompatible with the 1954 specification in two
   places, its `v|N` and its computed zero index; the p. 13 rule forbids that
   index whatever its sign; only the alarm after ten lines depends on the
   reconstruction's -0; nothing shows that the original machine failed
   (commits 505ddf0, 9b9c2e5).
9. **The largest-number correction and the main historical claims stand.**
   Agreed; nothing to change, except that the README now notes the page's
   "roughly" and labels the correction a slip. Codex also noted that the
   README makes no claim to have found these things first, but that "What the
   tests found" should not be read as one; the README now says that no search
   for earlier discussion has been made.
10. **The differential-equation results are credible, but the manual prints no
    results to check them against.** Accepted. Nothing claimed they were the
    manual's figures, but the README now says that the `.out` files are this
    reconstruction's output, kept as regression baselines, and that E-364
    prints no results for its programs.
11. **Spot-check of 20 transcriptions.** See above; the 3 undisclosed changes
    are finding 4.
12. **Build and test results.** Codex could not install the extension's
    dependencies or download VS Code without a network. Here, with one, all of
    it was run; see below.

Disputed: none of the findings. Finding 1 was accepted for the documentation
but not as a reason to change the run, for the reason given there.

## Test results after the changes

| Command | Result |
|---|---|
| `cargo build --release` | passed |
| `cargo test --release --workspace` | 38 unit, 6 command-line, 1 examples, 3 tutorial and 14 language-server tests passed |
| `node web/build.mjs && node web/test.mjs` | 40 examples, 40 files through the language server, 0 failed |
| `npm run build` and `npm test` in `editors/vscode` | 10 passed |
| `npm run test:vscode` in `editors/vscode` | passed (exit 0) |

The `.vsix` attached to the GitHub release was built before these changes and
has not been replaced.
