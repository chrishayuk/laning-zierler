//! The equations on the tape, read into statements.
//!
//! A program is a stream of characters, not of lines: an equation ends with a
//! comma, a PRINT with a period, and STOP ends the program. Line breaks are
//! only for the reader. The forms are those of E-364:
//!
//! ```text
//! [n] x = expression,          an equation (n is its number, 1 to 100)
//! [n] D x = expression,        a differential equation, dx/dt (section 15)
//! [n] PRINT a, b, x|3, y|i.    print the current values (section 4)
//!     SP n,  CP n,  SR n,  CR n,   jumps; n may be a letter (sections 5, 13)
//! [n] ASSIGN a|1000,           reserve a|1 to a|1000 on the drum (section 14)
//! [n] a|N = 2, 4, 6, 0,        values for a|1, a|2, ... on the drum
//! [n] a|N = 1 (.5) 2 (1) 4,    the same with increments in parentheses
//! [n] STOP                     the end of the program (section 6)
//! ```

use crate::lex::{lex, Span, Tok, Token};
use crate::num::MAX;

/// A variable: a letter, a superscript variable `x|5`, or one with a variable
/// index `x|n` (section 12).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Var {
    Letter(char),
    Super(char, u32),
    Indexed(char, char),
}

impl Var {
    pub fn letter(&self) -> char {
        match self {
            Var::Letter(c) | Var::Super(c, _) | Var::Indexed(c, _) => *c,
        }
    }
    /// The letters this reference reads: the letter itself if plain, and the
    /// index letter if indexed.
    pub fn index_letter(&self) -> Option<char> {
        match self {
            Var::Indexed(_, i) => Some(*i),
            _ => None,
        }
    }
}

impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Var::Letter(c) => write!(f, "{c}"),
            Var::Super(c, n) => write!(f, "{c}|{n}"),
            Var::Indexed(c, i) => write!(f, "{c}|{i}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarRef {
    pub var: Var,
    pub span: Span,
}

/// An expression: terms separated by plus and minus signs.
#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    /// Each term with its sign: false for +, true for -.
    pub terms: Vec<(bool, Term)>,
    pub span: Span,
}

/// A term: factors, each in the numerator or the denominator.
///
/// "The use of more than one division within a single mathematical term is
/// admissible ... Thus a/b/c is properly interpreted as the product ac divided
/// by b" (p. 4-5). A slash does not divide by the next factor alone: it moves
/// the factors after it to the other side of the fraction, until the next
/// slash. `cd/a(b+c)` is cd over a(b+c); `a/b/c` is ac over b.
#[derive(Clone, Debug, PartialEq)]
pub struct Term {
    pub factors: Vec<(bool, Factor)>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Factor {
    pub kind: FactorKind,
    /// The exponent typed after it, if any.
    pub power: Option<i32>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FactorKind {
    Number { text: String, value: f64 },
    Var(VarRef),
    Paren(Expr),
    /// F^k(argument): function subroutine k (section 11).
    Func { k: u32, arg: Expr, name_span: Span },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpKind {
    /// Go to the equation.
    SP,
    /// Go to it if the last computed quantity is negative.
    CP,
    /// Do that one equation, then come back.
    SR,
    /// Do it and come back, if the last computed quantity is negative.
    CR,
}

impl JumpKind {
    pub fn name(self) -> &'static str {
        match self {
            JumpKind::SP => "SP",
            JumpKind::CP => "CP",
            JumpKind::SR => "SR",
            JumpKind::CR => "CR",
        }
    }
    pub fn conditional(self) -> bool {
        matches!(self, JumpKind::CP | JumpKind::CR)
    }
    pub fn returns(self) -> bool {
        matches!(self, JumpKind::SR | JumpKind::CR)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Number(u32),
    Letter(char),
}

/// One value or run of values in an `a|N =` list.
#[derive(Clone, Debug, PartialEq)]
pub struct ListItem {
    /// The first value, then (increment, up to) pairs.
    pub start: f64,
    pub runs: Vec<(f64, f64)>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Equation { target: VarRef, expr: Expr },
    Derivative { target: VarRef, expr: Expr, d_span: Span },
    Print { items: Vec<VarRef> },
    Jump { kind: JumpKind, target: Target, target_span: Span },
    AssignDrum { letter: char, count: u32 },
    List { letter: char, items: Vec<ListItem> },
    Stop,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    /// The number the programmer typed in front of it, 1 to 100.
    pub label: Option<u32>,
    pub label_span: Option<Span>,
    pub kind: Kind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

/// The whole tape.
#[derive(Clone, Debug, Default)]
pub struct Tape {
    pub statements: Vec<Statement>,
    pub errors: Vec<ParseError>,
    pub tokens: Vec<Token>,
}

pub fn parse(src: &str) -> Tape {
    let tokens = lex(src);
    let toks: Vec<Token> = tokens.iter().filter(|t| t.tok != Tok::Note).cloned().collect();
    let mut p = Parser { toks, pos: 0, src_len: src.len(), errors: Vec::new() };
    let mut statements = Vec::new();
    while !p.at_end() {
        let start = p.pos;
        match p.statement() {
            Ok(s) => statements.push(s),
            Err(e) => {
                p.errors.push(e);
                p.recover(start);
            }
        }
    }
    Tape { statements, errors: p.errors, tokens }
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
    src_len: usize,
    errors: Vec<ParseError>,
}

type R<T> = Result<T, ParseError>;

fn err<T>(span: Span, message: impl Into<String>) -> R<T> {
    Err(ParseError { message: message.into(), span })
}

impl Parser {
    fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }
    fn peek_at(&self, k: usize) -> Option<&Tok> {
        self.toks.get(self.pos + k).map(|t| &t.tok)
    }
    fn span(&self) -> Span {
        match self.toks.get(self.pos) {
            Some(t) => t.span,
            None => Span::new(self.src_len, self.src_len),
        }
    }
    fn prev_span(&self) -> Span {
        if self.pos == 0 {
            return Span::new(0, 0);
        }
        self.toks[self.pos - 1].span
    }
    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        self.pos += 1;
        t
    }
    fn eat(&mut self, tok: &Tok) -> bool {
        if self.peek() == Some(tok) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn describe(&self) -> String {
        match self.peek() {
            None => "the end of the tape".into(),
            Some(t) => format!("\"{t}\""),
        }
    }

    /// After an error: skip to the end of the statement (past a comma or a
    /// period), always moving at least one token.
    fn recover(&mut self, start: usize) {
        if self.pos == start {
            self.pos += 1;
        }
        while let Some(t) = self.peek() {
            let stop = matches!(t, Tok::Comma | Tok::Period);
            self.pos += 1;
            if stop {
                // a list's values are separated by commas: skip numbers too
                break;
            }
        }
    }

    fn statement(&mut self) -> R<Statement> {
        let first = self.span();
        let mut label = None;
        let mut label_span = None;
        if let Some(Tok::Number(text)) = self.peek().cloned() {
            let sp = self.span();
            if text.contains('.') {
                return err(sp, format!("a statement cannot begin with the number {text}"));
            }
            let n: u64 = text.parse().unwrap_or(u64::MAX);
            if !(1..=100).contains(&n) {
                return err(sp, format!("equations are numbered 1 to 100, not {text}"));
            }
            self.pos += 1;
            label = Some(n as u32);
            label_span = Some(sp);
        }
        let kind = self.statement_kind(label.is_some())?;
        let span = first.to(self.prev_span());
        Ok(Statement { label, label_span, kind, span })
    }

    fn statement_kind(&mut self, numbered: bool) -> R<Kind> {
        match self.peek().cloned() {
            Some(Tok::Word(w)) => {
                let wspan = self.span();
                match w.as_str() {
                    "PRINT" => {
                        self.pos += 1;
                        self.print()
                    }
                    "STOP" => {
                        self.pos += 1;
                        Ok(Kind::Stop)
                    }
                    "SP" | "CP" | "SR" | "CR" => {
                        if numbered {
                            return err(wspan, format!("an {w} may not itself be assigned an equation number"));
                        }
                        self.pos += 1;
                        self.jump(&w)
                    }
                    "ASSIGN" => {
                        self.pos += 1;
                        self.assign_drum()
                    }
                    "D" => {
                        self.pos += 1;
                        let target = self.var_ref("after D")?;
                        if matches!(target.var, Var::Letter('t') | Var::Letter('h')) {
                            return err(target.span, "t is the independent variable and h its increment: neither may follow D");
                        }
                        self.expect(&Tok::Equals, "an equals sign")?;
                        let expr = self.expr(0)?;
                        self.end_equation()?;
                        Ok(Kind::Derivative { target, expr, d_span: wspan })
                    }
                    "F" => err(wspan, "an equation begins with the letter it gives a value to, not with a function"),
                    "N" => err(wspan, "N may only follow a vertical slash, as in g|N = 2, 4, 6,"),
                    _ => err(wspan, format!("\"{w}\" is not an instruction: the instructions are PRINT, SP, CP, SR, CR, ASSIGN, D and STOP; variables are lower-case letters")),
                }
            }
            Some(Tok::Letter(_)) => {
                // a|N = list, or an ordinary equation
                if self.peek_at(1) == Some(&Tok::Bar) && self.peek_at(2) == Some(&Tok::Word("N".into())) {
                    return self.list();
                }
                let target = self.var_ref("")?;
                self.expect(&Tok::Equals, "an equals sign")?;
                let expr = self.expr(0)?;
                self.end_equation()?;
                Ok(Kind::Equation { target, expr })
            }
            _ => err(self.span(), format!("expected an equation, PRINT, a jump or STOP, not {}", self.describe())),
        }
    }

    fn expect(&mut self, tok: &Tok, what: &str) -> R<()> {
        if self.eat(tok) {
            Ok(())
        } else {
            err(self.span(), format!("expected {what}, not {}", self.describe()))
        }
    }

    fn end_equation(&mut self) -> R<()> {
        if self.eat(&Tok::Comma) {
            return Ok(());
        }
        match self.peek() {
            Some(Tok::RParen) => err(self.span(), "a right parenthesis with no left parenthesis to close"),
            Some(Tok::Period) => err(self.span(), "every equation must end with a comma, not a period"),
            None => err(self.span(), "every equation must end with a comma"),
            _ => err(self.span(), format!("expected a comma to end the equation, not {}", self.describe())),
        }
    }

    /// A variable: x, x|5, x|⁵ or x|n.
    fn var_ref(&mut self, context: &str) -> R<VarRef> {
        let sp = self.span();
        let c = match self.peek() {
            Some(Tok::Letter(c)) => *c,
            _ => {
                let ctx = if context.is_empty() { String::new() } else { format!(" {context}") };
                return err(sp, format!("expected a lower-case letter{ctx}, not {}", self.describe()));
            }
        };
        self.pos += 1;
        if !self.eat(&Tok::Bar) {
            return Ok(VarRef { var: Var::Letter(c), span: sp });
        }
        let isp = self.span();
        let var = match self.peek().cloned() {
            Some(Tok::Number(text)) => {
                self.pos += 1;
                Var::Super(c, superscript_index(&text, isp)?)
            }
            Some(Tok::Super(n)) => {
                self.pos += 1;
                if !(0..=1023).contains(&n) {
                    return err(isp, "a superscript is an integer from 0 to 1023");
                }
                Var::Super(c, n as u32)
            }
            Some(Tok::Letter(i)) => {
                self.pos += 1;
                Var::Indexed(c, i)
            }
            Some(Tok::Word(w)) if w == "N" => {
                return err(isp, format!("{c}|N gives values to {c}|1, {c}|2, ... and must stand on the left of an equals sign at the start of an equation"));
            }
            _ => return err(isp, format!("after {c}| expected an integer from 0 to 1023 or a letter, not {}", self.describe())),
        };
        Ok(VarRef { var, span: sp.to(self.prev_span()) })
    }

    fn print(&mut self) -> R<Kind> {
        let mut items = Vec::new();
        loop {
            match self.peek() {
                Some(Tok::Letter(_)) => items.push(self.var_ref("")?),
                Some(Tok::Comma) => {
                    self.pos += 1;
                }
                Some(Tok::Period) => {
                    self.pos += 1;
                    break;
                }
                None => return err(self.span(), "every PRINT instruction must end with a period"),
                Some(Tok::Word(_) | Tok::Number(_)) if self.toks[self.pos - 1].tok == Tok::Comma => {
                    return err(self.prev_span(), "every PRINT instruction must end with a period, not a comma");
                }
                _ => {
                    return err(
                        self.span(),
                        format!("no computation may be performed in a PRINT instruction: only letters, commas and a period, not {}", self.describe()),
                    )
                }
            }
        }
        if items.is_empty() {
            return err(self.prev_span(), "PRINT of nothing");
        }
        Ok(Kind::Print { items })
    }

    fn jump(&mut self, w: &str) -> R<Kind> {
        let kind = match w {
            "SP" => JumpKind::SP,
            "CP" => JumpKind::CP,
            "SR" => JumpKind::SR,
            _ => JumpKind::CR,
        };
        let sp = self.span();
        let target = match self.peek().cloned() {
            Some(Tok::Number(text)) if !text.contains('.') => {
                self.pos += 1;
                let n: u64 = text.parse().unwrap_or(u64::MAX);
                if !(1..=100).contains(&n) {
                    return err(sp, format!("{w} is followed by an equation number from 1 to 100, not {text}"));
                }
                Target::Number(n as u32)
            }
            Some(Tok::Letter(c)) => {
                self.pos += 1;
                Target::Letter(c)
            }
            _ => return err(sp, format!("{w} is followed by an equation number or a lower-case letter, not {}", self.describe())),
        };
        if !self.eat(&Tok::Comma) {
            return err(self.span(), format!("the instruction {w} must be terminated by a comma"));
        }
        Ok(Kind::Jump { kind, target, target_span: sp })
    }

    fn assign_drum(&mut self) -> R<Kind> {
        let sp = self.span();
        let c = match self.peek() {
            Some(Tok::Letter(c)) => *c,
            _ => return err(sp, format!("ASSIGN is followed by a letter, a vertical slash and an integer, as in ASSIGN a|1000, not {}", self.describe())),
        };
        self.pos += 1;
        if !self.eat(&Tok::Bar) {
            return err(self.span(), format!("ASSIGN {c} needs |n: the number of variables to reserve"));
        }
        let isp = self.span();
        let n = match self.peek().cloned() {
            Some(Tok::Number(text)) => {
                self.pos += 1;
                superscript_index(&text, isp)?
            }
            Some(Tok::Super(n)) if (0..=1023).contains(&n) => {
                self.pos += 1;
                n as u32
            }
            _ => return err(isp, "ASSIGN reserves from 1 to 1023 variables"),
        };
        if n == 0 {
            return err(isp, "ASSIGN reserves from 1 to 1023 variables");
        }
        if !self.eat(&Tok::Comma) {
            return err(self.span(), "the instruction ASSIGN must be terminated by a comma");
        }
        Ok(Kind::AssignDrum { letter: c, count: n })
    }

    /// a|N = 2, 4, 6, 0,   or   a|N = 1.0 (.5) 2. (0.25) 2.5 (1) 4.5,
    fn list(&mut self) -> R<Kind> {
        let c = match self.bump().tok {
            Tok::Letter(c) => c,
            _ => unreachable!(),
        };
        self.pos += 2; // | N
        self.expect(&Tok::Equals, "an equals sign")?;
        let mut items = Vec::new();
        loop {
            let isp = self.span();
            let start = self.signed_number()?;
            let mut runs = Vec::new();
            while self.eat(&Tok::LParen) {
                let inc = self.signed_number()?;
                if !self.eat(&Tok::RParen) {
                    return err(self.span(), "expected a right parenthesis after the increment");
                }
                let upto = self.signed_number()?;
                if inc == 0.0 {
                    return err(isp, "an increment of zero");
                }
                runs.push((inc, upto));
            }
            if !self.eat(&Tok::Comma) {
                return err(self.span(), "each value, including the last, must be followed by a comma");
            }
            items.push(ListItem { start, runs, span: isp.to(self.prev_span()) });
            // Another value follows if the next token is a sign, or a number
            // that is not the number of the next equation (which is followed
            // by a letter or an instruction).
            let more = matches!(
                (self.peek(), self.peek_at(1)),
                (Some(Tok::Minus | Tok::Plus), _) | (Some(Tok::Number(_)), Some(Tok::Comma | Tok::LParen) | None)
            );
            if !more {
                break;
            }
        }
        Ok(Kind::List { letter: c, items })
    }

    fn signed_number(&mut self) -> R<f64> {
        let neg = if self.eat(&Tok::Minus) {
            true
        } else {
            self.eat(&Tok::Plus);
            false
        };
        let sp = self.span();
        match self.peek().cloned() {
            Some(Tok::Number(text)) => {
                self.pos += 1;
                let v = number_value(&text, sp)?;
                Ok(if neg { -v } else { v })
            }
            _ => err(sp, format!("expected a number, not {}", self.describe())),
        }
    }

    /// expression := ['+'|'-'] term { ('+'|'-') term }
    /// `depth` is the number of parentheses already open in this equation.
    fn expr(&mut self, depth: u32) -> R<Expr> {
        let start = self.span();
        let mut terms = Vec::new();
        let mut neg = false;
        if self.eat(&Tok::Minus) {
            neg = true;
        } else {
            self.eat(&Tok::Plus);
        }
        loop {
            let t = self.term(depth)?;
            terms.push((neg, t));
            if self.eat(&Tok::Plus) {
                neg = false;
            } else if self.eat(&Tok::Minus) {
                neg = true;
            } else {
                break;
            }
            if matches!(self.peek(), Some(Tok::Plus | Tok::Minus)) {
                return err(self.span(), "two signs together: put the second term in parentheses");
            }
        }
        Ok(Expr { terms, span: start.to(self.prev_span()) })
    }

    /// term := factor { ['·'] factor | '/' factor }
    fn term(&mut self, depth: u32) -> R<Term> {
        let start = self.span();
        let mut factors = Vec::new();
        let mut below = false;
        let mut need_factor = true;
        loop {
            match self.peek() {
                Some(Tok::Slash) => {
                    if need_factor {
                        return err(self.span(), "a slash with nothing before it");
                    }
                    self.pos += 1;
                    below = !below;
                    need_factor = true;
                    continue;
                }
                Some(Tok::Times) => {
                    if need_factor {
                        return err(self.span(), "a multiplication sign with nothing before it");
                    }
                    self.pos += 1;
                    need_factor = true;
                    continue;
                }
                _ => {}
            }
            if !self.starts_factor() {
                if need_factor {
                    return err(self.span(), format!("expected a number, a letter, a function or a parenthesis, not {}", self.describe()));
                }
                break;
            }
            let f = self.factor(depth)?;
            factors.push((below, f));
            need_factor = false;
        }
        Ok(Term { factors, span: start.to(self.prev_span()) })
    }

    fn starts_factor(&self) -> bool {
        match self.peek() {
            Some(Tok::Number(_) | Tok::Letter(_) | Tok::LParen) => true,
            Some(Tok::Word(w)) => w == "F",
            _ => false,
        }
    }

    fn factor(&mut self, depth: u32) -> R<Factor> {
        let sp = self.span();
        let kind = match self.peek().cloned() {
            Some(Tok::Number(text)) => {
                self.pos += 1;
                let value = number_value(&text, sp)?;
                FactorKind::Number { text, value }
            }
            Some(Tok::Letter(_)) => FactorKind::Var(self.var_ref("")?),
            Some(Tok::LParen) => {
                if depth >= 4 {
                    return err(sp, "no more than four parentheses may be open in a single equation at any one time");
                }
                self.pos += 1;
                let e = self.expr(depth + 1)?;
                if !self.eat(&Tok::RParen) {
                    return err(self.span(), format!("expected a right parenthesis, not {}", self.describe()));
                }
                FactorKind::Paren(e)
            }
            Some(Tok::Word(w)) if w == "F" => {
                self.pos += 1;
                let k = match self.peek().cloned() {
                    Some(Tok::Super(k)) if (1..=23).contains(&k) => k as u32,
                    Some(Tok::Super(k)) => return err(self.span(), format!("there are 23 function subroutines, F¹ to F²³; there is no F^{k}")),
                    _ => return err(self.span(), "F is followed by the number of a function subroutine as an exponent: F²(x) or F^2(x) is sin x"),
                };
                self.pos += 1;
                let name_span = sp.to(self.prev_span());
                if self.peek() != Some(&Tok::LParen) {
                    return err(self.span(), "the argument of a function subroutine goes in parentheses");
                }
                if depth >= 4 {
                    return err(self.span(), "no more than four parentheses may be open in a single equation at any one time");
                }
                self.pos += 1;
                let arg = self.expr(depth + 1)?;
                if !self.eat(&Tok::RParen) {
                    return err(self.span(), format!("expected a right parenthesis, not {}", self.describe()));
                }
                FactorKind::Func { k, arg, name_span }
            }
            _ => return err(sp, format!("expected a number, a letter, a function or a parenthesis, not {}", self.describe())),
        };
        let mut power = None;
        if let Some(Tok::Super(n)) = self.peek().cloned() {
            self.pos += 1;
            if n.abs() > 1023 {
                return err(self.prev_span(), "an exponent that large");
            }
            power = Some(n as i32);
        }
        Ok(Factor { kind, power, span: sp.to(self.prev_span()) })
    }
}

/// The index of a superscript variable, typed with ordinary digits: 0 to 1023.
fn superscript_index(text: &str, sp: Span) -> R<u32> {
    if text.contains('.') {
        return err(sp, "a superscript is an integer from 0 to 1023");
    }
    match text.parse::<u32>() {
        Ok(n) if n <= 1023 => Ok(n),
        _ => err(sp, "a superscript is an integer from 0 to 1023"),
    }
}

/// "The computer first reads in a number as an integer, placing the decimal
/// point only after the number is completely read in. This means that no
/// number is allowed which, without benefit of decimal point, would be
/// interpreted as an integer exceeding 2^63 - 2^39." (p. 6)
pub fn number_value(text: &str, sp: Span) -> R<f64> {
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return err(sp, "a point with no digits");
    }
    let as_integer: u128 = digits.parse().unwrap_or(u128::MAX);
    if as_integer as f64 > MAX || digits.len() > 30 {
        return err(sp, format!("{text}: read without its point this is an integer greater than 2^63 - 2^39"));
    }
    let after_point = match text.find('.') {
        Some(p) => text.len() - p - 1,
        None => 0,
    };
    let v: f64 = format!("{digits}e-{after_point}").parse().unwrap_or(0.0);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(src: &str) -> Kind {
        let t = parse(src);
        assert!(t.errors.is_empty(), "{:?}", t.errors);
        t.statements[0].kind.clone()
    }

    #[test]
    fn the_first_example() {
        let t = parse("a = 5,\ny = -6.3a,\nb = 0.0053 (a-y) /2ay,");
        assert!(t.errors.is_empty(), "{:?}", t.errors);
        assert_eq!(t.statements.len(), 3);
    }

    #[test]
    fn a_slash_moves_what_follows_to_the_other_side() {
        let Kind::Equation { expr, .. } = one("x = a/b/c,") else { panic!() };
        let sides: Vec<bool> = expr.terms[0].1.factors.iter().map(|f| f.0).collect();
        assert_eq!(sides, vec![false, true, false]); // a and c over b
        let Kind::Equation { expr, .. } = one("x = cd/a(b+c) + d,") else { panic!() };
        let sides: Vec<bool> = expr.terms[0].1.factors.iter().map(|f| f.0).collect();
        assert_eq!(sides, vec![false, false, true, true]);
    }

    #[test]
    fn four_parentheses_and_no_more() {
        assert!(parse("x = a(b + c(d + e(f + g(h + i)(j + k)^2)^-1)),").errors.is_empty());
        let e = parse("x = a(b + c(d + e(f + g(h + i(j + k)^2)^3)),").errors;
        assert!(e[0].message.contains("four parentheses"), "{e:?}");
    }

    #[test]
    fn labels_and_jumps() {
        let t = parse("3 x = 2,\n1 y = 0.5 (x + 2/x),\nSP 3,\n5y = ax,\nSTOP");
        assert!(t.errors.is_empty(), "{:?}", t.errors);
        assert_eq!(t.statements[0].label, Some(3));
        assert_eq!(t.statements[3].label, Some(5));
        assert!(parse("4 SP 3,").errors[0].message.contains("may not"));
    }

    #[test]
    fn lists() {
        let Kind::List { items, .. } = one("a|N = 1.0 (.5) 2. (0.25) 2.5 (1) 4.5,") else { panic!() };
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].runs, vec![(0.5, 2.0), (0.25, 2.5), (1.0, 4.5)]);
        let t = parse("g|N = 2, 4, 6, 0,\n1 n = n + 1,");
        assert!(t.errors.is_empty(), "{:?}", t.errors);
        let Kind::List { items, .. } = &t.statements[0].kind else { panic!() };
        assert_eq!(items.len(), 4);
        assert_eq!(t.statements[1].label, Some(1));
    }

    #[test]
    fn a_print_ended_with_a_comma() {
        let e = parse("a = 1,\nPRINT a,\nSTOP").errors;
        assert!(e[0].message.contains("must end with a period"), "{e:?}");
    }

    #[test]
    fn numbers_too_large() {
        assert!(parse("x = 9223314040000000000,").errors.is_empty());
        assert!(!parse("x = 9223372036854775807,").errors.is_empty());
        // the same number with a point is still too large: it is read as an integer first
        assert!(!parse("x = 922337203.6854775807,").errors.is_empty());
    }
}
