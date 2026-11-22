//! Reading a program without running it: what stops it translating, and what
//! E-364 warns about that the translator of 1954 would not have caught.
//!
//! The warnings are the manual's own:
//!
//! - "a letter variable must appear on the left hand side of an equation so
//!   that it may be assigned a definite value before it appears on the right
//!   hand side" (p. 3);
//! - "it is essential that all relevant auxiliary computation take place
//!   between the first and last D equations" (p. 19), with its example of a
//!   change that "would appear to be innocuous";
//! - CP after a difference that may be zero: "the CP may or may not behave
//!   like an SP since 0 may appear as +0 or -0" (p. 8);
//! - an index that may be a zero computed by arithmetic (p. 13).

use std::collections::{HashMap, HashSet};

use crate::lex::Span;
use crate::parse::{parse, Expr, FactorKind, JumpKind, Kind, Statement, Target, Var};
use crate::translate::{translate, Options};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Hint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
}

/// Every problem and warning, sorted by position.
pub fn check(src: &str, options: &Options) -> Vec<Diagnostic> {
    let (_, problems) = translate(src, options);
    let mut out: Vec<Diagnostic> =
        problems.into_iter().map(|p| Diagnostic { severity: Severity::Error, message: p.message, span: p.span }).collect();
    let tape = parse(src);
    warnings(&tape.statements, &mut out);
    fractional_exponents(&tape.tokens, &mut out);
    counted_down_indices(&tape.statements, &mut out);
    out.sort_by_key(|d| (d.span.start, d.severity));
    out.dedup_by(|a, b| a.span == b.span && a.message == b.message);
    out
}

/// Letters (plain) read by an expression, with where.
fn reads(e: &Expr, out: &mut Vec<(char, Span)>) {
    for (_, t) in &e.terms {
        for (_, f) in &t.factors {
            match &f.kind {
                FactorKind::Number { .. } => {}
                FactorKind::Var(v) => {
                    match &v.var {
                        Var::Letter(c) => out.push((*c, v.span)),
                        Var::Indexed(_, i) => out.push((*i, v.span)),
                        Var::Super(..) => {}
                    }
                }
                FactorKind::Paren(e) => reads(e, out),
                FactorKind::Func { arg, .. } => reads(arg, out),
            }
        }
    }
}

/// Every variable an expression reads, superscript ones included.
fn reads_vars(e: &Expr, out: &mut Vec<Var>) {
    for (_, t) in &e.terms {
        for (_, f) in &t.factors {
            match &f.kind {
                FactorKind::Number { .. } => {}
                FactorKind::Var(v) => out.push(v.var.clone()),
                FactorKind::Paren(e) => reads_vars(e, out),
                FactorKind::Func { arg, .. } => reads_vars(arg, out),
            }
        }
    }
}

fn warnings(statements: &[Statement], out: &mut Vec<Diagnostic>) {
    // Letters given values anywhere, and where first.
    let mut assigned_at: HashMap<char, usize> = HashMap::new();
    for (i, s) in statements.iter().enumerate() {
        if let Kind::Equation { target, .. } | Kind::Derivative { target, .. } = &s.kind {
            if let Var::Letter(c) = target.var {
                assigned_at.entry(c).or_insert(i);
            }
        }
    }
    let has_d = statements.iter().any(|s| matches!(s.kind, Kind::Derivative { .. }));
    if has_d {
        // t and h are given values by the system as it integrates, but the
        // programmer must start them.
        for c in ['t', 'h'] {
            if !assigned_at.contains_key(&c) {
                let span = statements.iter().find(|s| matches!(s.kind, Kind::Derivative { .. })).map(|s| s.span).unwrap_or_default();
                let what = if c == 't' { "the independent variable t" } else { "the increment h" };
                out.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{what} is never given a value: it starts at zero"),
                    span,
                });
            }
        }
    }

    // Used before any equation gives it a value.
    let mut warned: HashSet<char> = HashSet::new();
    for (i, s) in statements.iter().enumerate() {
        let mut r = Vec::new();
        match &s.kind {
            Kind::Equation { expr, target } | Kind::Derivative { expr, target, .. } => {
                reads(expr, &mut r);
                if let Var::Indexed(_, ix) = target.var {
                    r.push((ix, target.span));
                }
            }
            Kind::Print { items } => {
                for v in items {
                    match v.var {
                        Var::Letter(c) => r.push((c, v.span)),
                        Var::Indexed(_, ix) => r.push((ix, v.span)),
                        _ => {}
                    }
                }
            }
            Kind::Jump { target: Target::Letter(c), target_span, .. } => r.push((*c, *target_span)),
            _ => {}
        }
        for (c, span) in r {
            if warned.contains(&c) || (has_d && (c == 't' || c == 'h')) {
                continue;
            }
            match assigned_at.get(&c) {
                None => {
                    warned.insert(c);
                    out.push(Diagnostic {
                        severity: Severity::Warning,
                        message: format!("{c} is never given a value: a letter must appear on the left of an equation before it appears on the right"),
                        span,
                    });
                }
                Some(&first) if first >= i => {
                    // A jump back may make this fine; say so gently.
                    warned.insert(c);
                    out.push(Diagnostic {
                        severity: Severity::Warning,
                        message: format!("{c} is used here before the first equation that gives it a value (\"n = m - 1, m = 1, would be incorrect\", p. 3)"),
                        span,
                    });
                }
                _ => {}
            }
        }
    }

    // Auxiliary computation outside the D block.
    let first_d = statements.iter().position(|s| matches!(s.kind, Kind::Derivative { .. }));
    let last_d = statements.iter().rposition(|s| matches!(s.kind, Kind::Derivative { .. }));
    if let (Some(fd), Some(ld)) = (first_d, last_d) {
        let mut dependents: HashSet<Var> = HashSet::new();
        dependents.insert(Var::Letter('t'));
        for s in &statements[fd..=ld] {
            if let Kind::Derivative { target, .. } = &s.kind {
                dependents.insert(target.var.clone());
            }
        }
        // Variables read inside the block.
        let mut read_inside: HashSet<Var> = HashSet::new();
        for s in &statements[fd..=ld] {
            if let Kind::Equation { expr, .. } | Kind::Derivative { expr, .. } = &s.kind {
                let mut v = Vec::new();
                reads_vars(expr, &mut v);
                read_inside.extend(v);
            }
        }
        let assigned_inside: HashSet<Var> = statements[fd..=ld]
            .iter()
            .filter_map(|s| match &s.kind {
                Kind::Equation { target, .. } => Some(target.var.clone()),
                _ => None,
            })
            .collect();
        for (i, s) in statements.iter().enumerate() {
            if i >= fd && i <= ld {
                continue;
            }
            if let Kind::Equation { target, expr } = &s.kind {
                let mut v = Vec::new();
                reads_vars(expr, &mut v);
                let depends = v.iter().any(|x| dependents.contains(x));
                if depends && read_inside.contains(&target.var) && !assigned_inside.contains(&target.var) {
                    out.push(Diagnostic {
                        severity: Severity::Warning,
                        message: format!(
                            "{} is computed from the solution outside the D equations but used inside them: each step goes through the D equations four times, and {} is not recomputed between passes. \"It is essential that all relevant auxiliary computation take place between the first and last D equations\" (p. 19)",
                            target.var, target.var
                        ),
                        span: s.span,
                    });
                }
            }
        }
    }

    // CP or CR straight after a difference of whole numbers.
    let mut prev_eq: Option<&Statement> = None;
    for s in statements {
        match &s.kind {
            Kind::Jump { kind: JumpKind::CP | JumpKind::CR, .. } => {
                if let Some(Statement { kind: Kind::Equation { expr, target }, .. }) = prev_eq {
                    if let Some(k) = whole_number_difference(expr) {
                        out.push(Diagnostic {
                            severity: Severity::Hint,
                            message: format!(
                                "if {} comes out exactly zero, \"the CP may or may not behave like an SP since 0 may appear as +0 or -0\" (p. 8). The manual's own loops compare with a half: n - {}.5, not n - {k}",
                                target.var,
                                k.saturating_sub(1)
                            ),
                            span: s.span,
                        });
                    }
                }
            }
            Kind::Equation { .. } | Kind::Derivative { .. } => prev_eq = Some(s),
            _ => {}
        }
    }
}

/// `x^1.5` is read as x¹ times .5: an exponent is upper-case digits only, and
/// the point and the 5 after it are lower case, a number of their own.
fn fractional_exponents(tokens: &[crate::lex::Token], out: &mut Vec<Diagnostic>) {
    use crate::lex::Tok;
    for w in tokens.windows(2) {
        if let (Tok::Super(n), Tok::Number(text)) = (&w[0].tok, &w[1].tok) {
            if w[0].span.end == w[1].span.start && text.starts_with('.') {
                out.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!(
                        "this is read as the power {n} times {text}: exponents are whole numbers (\"positive (unsigned) and negative integral exponents\", p. 5)"
                    ),
                    span: w[0].span.to(w[1].span),
                });
            }
        }
    }
}

/// An index letter that is counted down by arithmetic (i = i - 1): if it comes
/// to zero that way, page 13 does not permit it as an index.
fn counted_down_indices(statements: &[Statement], out: &mut Vec<Diagnostic>) {
    let mut index_letters: HashSet<char> = HashSet::new();
    let mut note = |v: &Var| {
        if let Var::Indexed(_, i) = v {
            index_letters.insert(*i);
        }
    };
    for s in statements {
        match &s.kind {
            Kind::Equation { target, expr } | Kind::Derivative { target, expr, .. } => {
                note(&target.var);
                let mut v = Vec::new();
                reads_vars(expr, &mut v);
                v.iter().for_each(&mut note);
            }
            Kind::Print { items } => items.iter().for_each(|r| note(&r.var)),
            _ => {}
        }
    }
    for s in statements {
        let Kind::Equation { target, expr } = &s.kind else { continue };
        let Var::Letter(c) = target.var else { continue };
        if !index_letters.contains(&c) || expr.terms.len() != 2 || !expr.terms[1].0 {
            continue;
        }
        let reads_itself = matches!(expr.terms[0].1.factors.as_slice(),
            [(false, f)] if f.power.is_none() && matches!(&f.kind, FactorKind::Var(v) if v.var == Var::Letter(c)));
        if reads_itself {
            out.push(Diagnostic {
                severity: Severity::Hint,
                message: format!(
                    "{c} is used as an index and counted down by arithmetic. If it reaches zero this way it may not be used as one: zero is permissible \"if assigned by the specific equation {c} = 0,\" (p. 13)"
                ),
                span: s.span,
            });
        }
    }
}

/// `x - 10` or `10 - x`, with a whole number: the constant, if so.
fn whole_number_difference(e: &Expr) -> Option<u64> {
    if e.terms.len() != 2 {
        return None;
    }
    let constant = |t: &crate::parse::Term| -> Option<f64> {
        match t.factors.as_slice() {
            [(false, f)] if f.power.is_none() => match &f.kind {
                FactorKind::Number { value, .. } => Some(*value),
                _ => None,
            },
            _ => None,
        }
    };
    let c = constant(&e.terms[1].1).or_else(|| constant(&e.terms[0].1))?;
    if e.terms[1].0 && c.fract() == 0.0 && c > 0.0 {
        Some(c as u64)
    } else {
        None
    }
}

/// Line and column (both from 0, column in characters) of a byte offset.
pub fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let before = &src[..offset.min(src.len())];
    let line = before.matches('\n').count();
    let col = before.rsplit('\n').next().unwrap_or("").chars().count();
    (line, col)
}

/// `file:line:col: severity: message`, one per line.
pub fn render(src: &str, file: &str, diags: &[Diagnostic]) -> String {
    let mut s = String::new();
    for d in diags {
        let (l, c) = line_col(src, d.span.start);
        let sev = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Hint => "hint",
        };
        s.push_str(&format!("{file}:{}:{}: {sev}: {}\n", l + 1, c + 1, d.message));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msgs(src: &str) -> Vec<String> {
        check(src, &Options::default()).into_iter().map(|d| d.message).collect()
    }

    #[test]
    fn the_innocuous_change() {
        // p. 19: a is computed before the D equations and used in them
        let src = "t = 2,\nh = 0.5,\ny|1 = 0,\ny|2 = 0,\n1 PRINT t, y|1, y|2.\ne = 9.7 - t,\nCP 3,\n2 a = y|2 + 1,\nD y|1 = a,\nD y|2 = - y|1,\ne = -e,\nCP 2,\nSP 1,\n3 STOP";
        let m = msgs(src);
        assert!(m.iter().any(|m| m.contains("auxiliary computation")), "{m:?}");
    }

    #[test]
    fn a_fractional_exponent() {
        let m = msgs("x = 2^1.5,\nSTOP");
        assert!(m.iter().any(|m| m.contains("power 1 times .5")), "{m:?}");
        assert!(!msgs("x = 2^1 .5,\nSTOP").iter().any(|m| m.contains("times .5")));
    }

    #[test]
    fn an_index_counted_down() {
        let m = msgs("i = 3,\n1 i = i - 1,\nx = a|i,\ne = -0.5 - i,\nCP 1,\nSTOP");
        assert!(m.iter().any(|m| m.contains("counted down")), "{m:?}");
    }

    #[test]
    fn used_before_assigned() {
        let m = msgs("n = m - 1,\nm = 1,\nSTOP");
        assert!(m.iter().any(|m| m.contains("m is used here before")), "{m:?}");
    }

    #[test]
    fn cp_after_whole_difference() {
        let m = msgs("n = 0,\n1 n = n + 1,\ne = n - 10,\nCP 1,\nSTOP");
        assert!(m.iter().any(|m| m.contains("may or may not")), "{m:?}");
        let m = msgs("n = 0,\n1 n = n + 1,\ne = n - 9.5,\nCP 1,\nSTOP");
        assert!(!m.iter().any(|m| m.contains("may or may not")), "{m:?}");
    }
}
