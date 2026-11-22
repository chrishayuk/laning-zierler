//! Translation: statements into blocks of interpretive orders.
//!
//! "The source code was translated into blocks of subroutine calls, stored on
//! the drum, and after being transferred to core storage (one equation's worth
//! at a time) these subroutines invoked the standard floating-point
//! interpretive routines on the Whirlwind" (Knuth and Trabb Pardo, 1976).
//!
//! The orders here are this reconstruction's, not Whirlwind's: a small stack
//! code, one order per floating-point routine called. What they keep from the
//! original is the shape: each equation is translated once, before anything
//! runs, into its own block; the blocks are what is executed; and every
//! equation is numbered, by the programmer (1 to 100) or by the translator
//! (101 to 200, "in the order in which they are written").

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::lex::Span;
use crate::num::{Num, ZeroModel};
use crate::parse::{parse, Expr, Factor, FactorKind, JumpKind, Kind, Statement, Target, Term, Var, VarRef};

/// One interpretive order.
#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    /// Push a constant.
    Const(Num),
    /// Push the value of a variable.
    Load(Var),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    /// Raise to an integral power.
    Pow(i32),
    /// Call function subroutine F^k.
    Func(u32),
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Order::Const(n) => write!(f, "const {n}"),
            Order::Load(v) => write!(f, "load  {v}"),
            Order::Add => write!(f, "add"),
            Order::Sub => write!(f, "sub"),
            Order::Mul => write!(f, "mul"),
            Order::Div => write!(f, "div"),
            Order::Neg => write!(f, "neg"),
            Order::Pow(n) => write!(f, "power {}", crate::lex::superscript(*n as i64)),
            Order::Func(k) => write!(f, "call  F{} ({})", crate::lex::superscript(*k as i64), crate::num::FUNCTIONS[*k as usize - 1].1),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Exec {
    /// Compute, and store in the variable.
    Equation { orders: Vec<Order>, target: VarRef },
    /// Compute dy/dt for the differential equation with this slot.
    Derivative { orders: Vec<Order>, target: VarRef, slot: usize },
    Print(Vec<VarRef>),
    Jump { kind: JumpKind, target: Target },
    /// ASSIGN and a|N: done during translation; nothing happens when reached.
    Declaration,
    Stop,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Equation {
    /// The number of the equation: the programmer's (1-100), or the
    /// translator's (101-200). Jumps, ASSIGN and a|N have none.
    pub number: Option<u32>,
    pub labelled: bool,
    pub exec: Exec,
    /// The equation calls a function subroutine (see the first temporary
    /// restriction of section 18).
    pub uses_function: bool,
    pub span: Span,
}

/// Variables the translator put on the drum: from ASSIGN a|n (no values) or
/// a|N = ... (with values).
#[derive(Clone, Debug, PartialEq)]
pub struct DrumGroup {
    pub letter: char,
    pub values: Vec<Num>,
    pub span: Span,
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    /// How a zero gets its sign.
    pub zero: ZeroModel,
    /// Apply the "Temporary Rules and Restrictions" of section 18, as of
    /// January 1954: no SR or CR to an equation that uses a function
    /// subroutine, and no a|N.
    pub temporary_rules: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub message: String,
    pub span: Span,
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub equations: Vec<Equation>,
    /// The programmer's numbers: equation number to its index.
    pub labels: HashMap<u32, usize>,
    pub drum: Vec<DrumGroup>,
    /// Index of the first and last differential equations, if any.
    pub first_d: Option<usize>,
    pub last_d: Option<usize>,
    pub d_count: usize,
    /// Superscript variables (in core) that the program names with a fixed
    /// superscript.
    pub fixed_supers: usize,
}

/// Space for superscript variables in MS (core): "room for a total of 250".
pub const CORE_SUPERS: usize = 250;
/// The drum: "a maximum of 3072", shared with the equations.
pub const DRUM_VARIABLES: usize = 3072;

impl Program {
    /// How many superscript variables may be in core: 250, less two for each
    /// differential equation, less one for each group on the drum.
    pub fn core_super_limit(&self) -> usize {
        CORE_SUPERS.saturating_sub(2 * self.d_count + self.drum.len())
    }

    /// The drum space taken by the translated equations, in variables. The
    /// manual says only that "the drum is shared by the equations and the drum
    /// variables"; how much an equation took is not known. This counts one
    /// variable's worth of drum for every two orders, which is a guess.
    pub fn drum_for_equations(&self) -> usize {
        let orders: usize = self
            .equations
            .iter()
            .map(|e| match &e.exec {
                Exec::Equation { orders, .. } | Exec::Derivative { orders, .. } => orders.len() + 1,
                _ => 1,
            })
            .sum();
        orders.div_ceil(2)
    }

    pub fn by_number(&self, n: u32) -> Option<&Equation> {
        self.labels.get(&n).map(|&i| &self.equations[i])
    }
}

/// Translate a tape. Problems are what stopped translation; the program is
/// usable only if there are none.
pub fn translate(src: &str, options: &Options) -> (Program, Vec<Problem>) {
    let tape = parse(src);
    let mut problems: Vec<Problem> =
        tape.errors.iter().map(|e| Problem { message: e.message.clone(), span: e.span }).collect();
    let program = build(&tape.statements, options, &mut problems);
    (program, problems)
}

fn build(statements: &[Statement], options: &Options, problems: &mut Vec<Problem>) -> Program {
    let mut p = Program::default();
    let mut next_auto = 101u32;
    let mut supers: HashSet<(char, u32)> = HashSet::new();
    let mut drum_letters: HashMap<char, usize> = HashMap::new();
    let mut auto_overflow_reported = false;

    for s in statements {
        let mut number = None;
        let countable = !matches!(s.kind, Kind::Jump { .. } | Kind::AssignDrum { .. } | Kind::List { .. });
        if let Some(l) = s.label {
            if p.labels.contains_key(&l) {
                problems.push(Problem { message: format!("two equations are numbered {l}"), span: s.label_span.unwrap_or(s.span) });
            } else {
                p.labels.insert(l, p.equations.len());
            }
            number = Some(l);
        } else if countable {
            if next_auto > 200 {
                if !auto_overflow_reported {
                    problems.push(Problem {
                        message: "a maximum of 100 numbered equations and 100 unnumbered equations may be used in any one problem: this is the 101st unnumbered one".into(),
                        span: s.span,
                    });
                    auto_overflow_reported = true;
                }
            } else {
                number = Some(next_auto);
            }
            next_auto += 1;
        }
        let mut uses_function = false;
        let exec = match &s.kind {
            Kind::Equation { target, expr } => {
                note_super(&target.var, &mut supers);
                let mut orders = Vec::new();
                compile_expr(expr, &mut orders, &mut supers, options, problems);
                uses_function = orders.iter().any(|o| matches!(o, Order::Func(_)));
                Exec::Equation { orders, target: target.clone() }
            }
            Kind::Derivative { target, expr, .. } => {
                note_super(&target.var, &mut supers);
                let mut orders = Vec::new();
                compile_expr(expr, &mut orders, &mut supers, options, problems);
                uses_function = orders.iter().any(|o| matches!(o, Order::Func(_)));
                let slot = p.d_count;
                p.d_count += 1;
                if p.first_d.is_none() {
                    p.first_d = Some(p.equations.len());
                }
                p.last_d = Some(p.equations.len());
                Exec::Derivative { orders, target: target.clone(), slot }
            }
            Kind::Print { items } => {
                for v in items {
                    note_super(&v.var, &mut supers);
                }
                Exec::Print(items.clone())
            }
            Kind::Jump { kind, target, .. } => Exec::Jump { kind: *kind, target: target.clone() },
            Kind::AssignDrum { letter, count } => {
                add_drum(&mut p, &mut drum_letters, *letter, vec![Num::ZERO; *count as usize], s.span, problems);
                Exec::Declaration
            }
            Kind::List { letter, items } => {
                if options.temporary_rules {
                    problems.push(Problem {
                        message: "\"The x|N is not yet in operation\" (E-364, section 18, as of January 1954)".into(),
                        span: s.span,
                    });
                }
                let values = expand_list(items, options.zero, s.span, problems);
                add_drum(&mut p, &mut drum_letters, *letter, values, s.span, problems);
                Exec::Declaration
            }
            Kind::Stop => Exec::Stop,
        };
        p.equations.push(Equation { number, labelled: s.label.is_some(), exec, uses_function, span: s.span });
    }

    // Superscript variables that fall in a drum group are on the drum.
    p.fixed_supers = supers
        .iter()
        .filter(|(c, n)| !drum_letters.get(c).is_some_and(|&g| *n >= 1 && (*n as usize) <= p.drum[g].values.len()))
        .count();
    if p.fixed_supers > p.core_super_limit() {
        let span = statements.first().map(|s| s.span).unwrap_or_default();
        problems.push(Problem {
            message: format!(
                "storage exceeded: the program names {} superscript variables and there is room in MS for {} (250, less 2 for each of {} D equations and 1 for each of {} groups on the drum)",
                p.fixed_supers,
                p.core_super_limit(),
                p.d_count,
                p.drum.len()
            ),
            span,
        });
    }
    let drum_vars: usize = p.drum.iter().map(|g| g.values.len()).sum();
    if drum_vars + p.drum_for_equations() > DRUM_VARIABLES {
        let span = p.drum.last().map(|g| g.span).unwrap_or_default();
        problems.push(Problem {
            message: format!(
                "storage exceeded: {drum_vars} variables on the drum and the equations (about {} variables' worth) do not fit in the drum's {DRUM_VARIABLES}",
                p.drum_for_equations()
            ),
            span,
        });
    }

    // Jumps to numbers that no equation has.
    for e in &p.equations {
        if let Exec::Jump { kind, target: Target::Number(n) } = &e.exec {
            match p.labels.get(n) {
                None => problems.push(Problem { message: format!("{} {n}: there is no equation numbered {n}", kind.name()), span: e.span }),
                Some(&i) => {
                    if options.temporary_rules && kind.returns() && p.equations[i].uses_function {
                        problems.push(Problem {
                            message: format!(
                                "\"An SR or CR instruction may not refer to an equation that makes use of a subroutine\" (E-364, section 18): equation {n} calls a function"
                            ),
                            span: e.span,
                        });
                    }
                }
            }
        }
    }

    // STOP: "must be placed at the end of every program" and "may appear only
    // at the end".
    let stops: Vec<usize> = p.equations.iter().enumerate().filter(|(_, e)| e.exec == Exec::Stop).map(|(i, _)| i).collect();
    match stops.last() {
        None => {
            let end = statements.last().map(|s| Span::new(s.span.end, s.span.end)).unwrap_or_default();
            if problems.is_empty() {
                problems.push(Problem { message: "the instruction STOP must be placed at the end of every program".into(), span: end });
            }
        }
        Some(&last) => {
            for &i in &stops[..stops.len() - 1] {
                problems.push(Problem { message: "STOP may appear only at the end of a program".into(), span: p.equations[i].span });
            }
            if last != p.equations.len() - 1 {
                problems.push(Problem { message: "STOP may appear only at the end of a program".into(), span: p.equations[last].span });
            }
        }
    }
    p
}

fn note_super(v: &Var, supers: &mut HashSet<(char, u32)>) {
    if let Var::Super(c, n) = v {
        supers.insert((*c, *n));
    }
}

fn add_drum(
    p: &mut Program,
    drum_letters: &mut HashMap<char, usize>,
    letter: char,
    values: Vec<Num>,
    span: Span,
    problems: &mut Vec<Problem>,
) {
    if values.len() > 1023 {
        problems.push(Problem { message: format!("{} values for {letter}|1 to {letter}|1023", values.len()), span });
    }
    if drum_letters.contains_key(&letter) {
        problems.push(Problem { message: format!("{letter}| has already been assigned to the drum"), span });
        return;
    }
    drum_letters.insert(letter, p.drum.len());
    p.drum.push(DrumGroup { letter, values, span });
}

/// 1.0 (.5) 2. (0.25) 2.5  is 1, 1.5, 2, 2.25, 2.5.
fn expand_list(items: &[crate::parse::ListItem], zero: ZeroModel, span: Span, problems: &mut Vec<Problem>) -> Vec<Num> {
    let mut out = Vec::new();
    for item in items {
        let mut v = match Num::round(item.start) {
            Ok(v) => v,
            Err(e) => {
                problems.push(Problem { message: e.to_string(), span: item.span });
                return out;
            }
        };
        out.push(v);
        for &(inc, upto) in &item.runs {
            let steps = ((upto - v.value()) / inc).round();
            if steps < 0.0 || steps > 1023.0 {
                problems.push(Problem { message: format!("an increment of {inc} does not go from {} to {upto}", v.value()), span: item.span });
                return out;
            }
            let inc = Num::from_f64(inc);
            for _ in 0..steps as usize {
                v = match v.add(inc, zero) {
                    Ok(v) => v,
                    Err(e) => {
                        problems.push(Problem { message: e.to_string(), span });
                        return out;
                    }
                };
                out.push(v);
            }
        }
    }
    out
}

fn compile_expr(e: &Expr, out: &mut Vec<Order>, supers: &mut HashSet<(char, u32)>, options: &Options, problems: &mut Vec<Problem>) {
    for (i, (neg, term)) in e.terms.iter().enumerate() {
        compile_term(term, out, supers, options, problems);
        if i == 0 {
            if *neg {
                out.push(Order::Neg);
            }
        } else {
            out.push(if *neg { Order::Sub } else { Order::Add });
        }
    }
}

/// The numerator's factors multiplied together, then the denominator's, then
/// one division.
fn compile_term(t: &Term, out: &mut Vec<Order>, supers: &mut HashSet<(char, u32)>, options: &Options, problems: &mut Vec<Problem>) {
    let mut first = true;
    for (below, f) in &t.factors {
        if !below {
            compile_factor(f, out, supers, options, problems);
            if !first {
                out.push(Order::Mul);
            }
            first = false;
        }
    }
    let mut first = true;
    for (below, f) in &t.factors {
        if *below {
            compile_factor(f, out, supers, options, problems);
            if !first {
                out.push(Order::Mul);
            }
            first = false;
        }
    }
    if !first {
        out.push(Order::Div);
    }
}

fn compile_factor(f: &Factor, out: &mut Vec<Order>, supers: &mut HashSet<(char, u32)>, options: &Options, problems: &mut Vec<Problem>) {
    match &f.kind {
        FactorKind::Number { value, .. } => match Num::round(*value) {
            Ok(n) => out.push(Order::Const(n)),
            Err(e) => problems.push(Problem { message: e.to_string(), span: f.span }),
        },
        FactorKind::Var(v) => {
            note_super(&v.var, supers);
            out.push(Order::Load(v.var.clone()));
        }
        FactorKind::Paren(e) => compile_expr(e, out, supers, options, problems),
        FactorKind::Func { k, arg, .. } => {
            compile_expr(arg, out, supers, options, problems);
            out.push(Order::Func(*k));
        }
    }
    if let Some(n) = f.power {
        out.push(Order::Pow(n));
    }
}

/// The translation as text: each equation's number, its source and its block.
pub fn listing(src: &str, program: &Program) -> String {
    let mut s = String::new();
    for e in &program.equations {
        let text: String = src[e.span.start..e.span.end].split_whitespace().collect::<Vec<_>>().join(" ");
        let num = match e.number {
            Some(n) => format!("{n:>3}"),
            None => "   ".into(),
        };
        s.push_str(&format!("{num}  {text}\n"));
        match &e.exec {
            Exec::Equation { orders, target } => {
                for o in orders {
                    s.push_str(&format!("       {o}\n"));
                }
                s.push_str(&format!("       store {}\n", target.var));
            }
            Exec::Derivative { orders, target, slot } => {
                for o in orders {
                    s.push_str(&format!("       {o}\n"));
                }
                s.push_str(&format!("       deriv d{}/dt -> slot {slot}\n", target.var));
            }
            Exec::Print(items) => {
                let names: Vec<String> = items.iter().map(|v| v.var.to_string()).collect();
                s.push_str(&format!("       print {}\n", names.join(", ")));
            }
            Exec::Jump { kind, target } => {
                let t = match target {
                    Target::Number(n) => n.to_string(),
                    Target::Letter(c) => format!("the value of {c}"),
                };
                let what = match kind {
                    JumpKind::SP => format!("go to {t}"),
                    JumpKind::CP => format!("if negative, go to {t}"),
                    JumpKind::SR => format!("do {t} and come back"),
                    JumpKind::CR => format!("if negative, do {t} and come back"),
                };
                s.push_str(&format!("       {what}\n"));
            }
            Exec::Declaration => s.push_str("       (done during translation: on the drum)\n"),
            Exec::Stop => s.push_str("       stop\n"),
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(src: &str) -> Program {
        let (p, problems) = translate(src, &Options::default());
        assert!(problems.is_empty(), "{problems:?}");
        p
    }

    #[test]
    fn numbering() {
        let p = ok("a = 5,\n3 x = 2,\nSP 3,\nPRINT a.\nSTOP");
        let nums: Vec<Option<u32>> = p.equations.iter().map(|e| e.number).collect();
        assert_eq!(nums, vec![Some(101), Some(3), None, Some(102), Some(103)]);
    }

    #[test]
    fn a_over_b_over_c() {
        let p = ok("x = a/b/c,\nSTOP");
        let Exec::Equation { orders, .. } = &p.equations[0].exec else { panic!() };
        assert_eq!(
            orders,
            &vec![Order::Load(Var::Letter('a')), Order::Load(Var::Letter('c')), Order::Mul, Order::Load(Var::Letter('b')), Order::Div]
        );
    }

    #[test]
    fn lists_expand() {
        let p = ok("a|N = 1.0 (.5) 2. (0.25) 2.5 (1) 4.5,\nSTOP");
        let v: Vec<f64> = p.drum[0].values.iter().map(|n| n.value()).collect();
        assert_eq!(v, vec![1.0, 1.5, 2.0, 2.25, 2.5, 3.5, 4.5]);
    }

    #[test]
    fn stop_is_required_and_last() {
        assert!(!translate("a = 1,", &Options::default()).1.is_empty());
        assert!(!translate("STOP\na = 1,", &Options::default()).1.is_empty());
    }

    #[test]
    fn the_temporary_rules() {
        let src = "1 x = F^1(y),\nSR 1,\ng|N = 1, 2,\nSTOP";
        assert!(translate(src, &Options::default()).1.is_empty());
        let rules = Options { temporary_rules: true, ..Options::default() };
        assert_eq!(translate(src, &rules).1.len(), 2);
    }
}
