//! The algebraic system of J. H. Laning Jr. and N. Zierler for Whirlwind I,
//! as described in MIT Instrumentation Laboratory Engineering Memorandum
//! E-364, "A Program for Translation of Mathematical Equations for
//! Whirlwind I" (January 1954).
//!
//! A program is read ([`parse`]), translated into blocks of interpretive
//! orders ([`translate`]), and run ([`machine`]). [`check`] reads a program
//! without running it, for the command line and the language server.

pub mod check;
pub mod lex;
pub mod machine;
pub mod num;
pub mod parse;
pub mod translate;

/// Translate and run a tape with the default options: the typed page, or the
/// problems that stopped translation.
pub fn run_source(src: &str) -> Result<machine::Outcome, Vec<translate::Problem>> {
    let (program, problems) = translate::translate(src, &translate::Options::default());
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(machine::run(&program, &machine::RunOptions::default()))
}
