//! Reading the tape: Flexowriter characters into tokens.
//!
//! The rules are those of E-364, sections 2, 8 and 9:
//!
//! - space, back space, tabulate and carriage return are ignored, "except when
//!   they occur between the digits of a number"; there they are "interpreted
//!   as a multiplication sign", so `2 3` is 6 and `23` is twenty-three;
//! - the nullify character is always ignored (here it is DEL, 0x7F);
//! - `·` (a colon with its lower dot filed off) or the colon itself means
//!   times;
//! - upper-case digits and the upper-case minus are exponents.
//!
//! A Flexowriter typed superscripts; a keyboard today does not. So an
//! exponent may be written `x²`, `x⁻³` or `x^2`, `x^-3`, and `F²(x)` or
//! `F^2(x)`. The index of a superscript variable, `x|⁵`, may be written
//! `x|5`. `*` is accepted for `·`. A `#` starts a note to the end of the line;
//! notes are not on the tape.

use std::fmt;

/// Byte offsets into the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }
    pub fn to(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    /// Digits with at most one point, as typed: `0.0053`, `012`, `.5`, `2.`.
    Number(String),
    /// A lower-case letter: a variable.
    Letter(char),
    /// A run of upper-case letters: PRINT, STOP, SP, CP, SR, CR, ASSIGN, D, F, N.
    Word(String),
    /// An exponent: upper-case digits, perhaps with the upper-case minus.
    Super(i64),
    /// The vertical slash of a superscript variable.
    Bar,
    Plus,
    Minus,
    /// `·`, `:` or `*`, or the multiplication implied by a space between digits.
    Times,
    Slash,
    LParen,
    RParen,
    Comma,
    Period,
    Equals,
    /// A note: `#` to the end of the line. Kept for the editor; not on the tape.
    Note,
    /// A character the Flexowriter tape could not carry.
    Bad(char),
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Number(s) => write!(f, "{s}"),
            Tok::Letter(c) => write!(f, "{c}"),
            Tok::Word(w) => write!(f, "{w}"),
            Tok::Super(n) => write!(f, "{}", superscript(*n)),
            Tok::Bar => write!(f, "|"),
            Tok::Plus => write!(f, "+"),
            Tok::Minus => write!(f, "-"),
            Tok::Times => write!(f, "·"),
            Tok::Slash => write!(f, "/"),
            Tok::LParen => write!(f, "("),
            Tok::RParen => write!(f, ")"),
            Tok::Comma => write!(f, ","),
            Tok::Period => write!(f, "."),
            Tok::Equals => write!(f, "="),
            Tok::Note => write!(f, "#"),
            Tok::Bad(c) => write!(f, "{c}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
    /// The multiplication was implied by a space between two numbers (it has
    /// no characters of its own).
    pub implied: bool,
}

/// Write an integer as superscript characters.
pub fn superscript(n: i64) -> String {
    let mut s = String::new();
    if n < 0 {
        s.push('⁻');
    }
    for d in n.unsigned_abs().to_string().chars() {
        s.push(SUPERS[d.to_digit(10).unwrap() as usize]);
    }
    s
}

const SUPERS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

fn super_digit(c: char) -> Option<u32> {
    SUPERS.iter().position(|&s| s == c).map(|p| p as u32)
}

/// Characters ignored except between digits.
fn is_blank(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{8}')
}

/// Read the whole tape. Never fails: a character that cannot be read becomes
/// [`Tok::Bad`] and the parser reports it.
pub fn lex(src: &str) -> Vec<Token> {
    let chars: Vec<(usize, char)> = src.char_indices().collect();
    let end_of = |i: usize| -> usize { chars.get(i).map(|&(o, _)| o).unwrap_or(src.len()) };
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0;
    // Blank characters seen since the last token, and where they began.
    let mut blank_from: Option<usize> = None;
    while i < chars.len() {
        let (off, c) = chars[i];
        if is_blank(c) {
            blank_from.get_or_insert(off);
            i += 1;
            continue;
        }
        if c == '\u{7f}' {
            // the nullify character: always ignored
            i += 1;
            continue;
        }
        if c == '#' {
            let mut j = i;
            while j < chars.len() && chars[j].1 != '\n' {
                j += 1;
            }
            out.push(Token { tok: Tok::Note, span: Span::new(off, end_of(j)), implied: false });
            i = j;
            continue;
        }
        let had_blank = blank_from.take();
        // A number: digits and at most one point. A point not followed by a
        // digit and not preceded by one is a period.
        let starts_number = c.is_ascii_digit()
            || (c == '.' && chars.get(i + 1).is_some_and(|&(_, d)| d.is_ascii_digit() || d == '\u{7f}'));
        if starts_number {
            // "Any of these characters inserted between digits is interpreted
            // as a multiplication sign": a number after blanks after a number.
            if let (Some(from), Some(last)) = (had_blank, out.last()) {
                if matches!(last.tok, Tok::Number(_)) {
                    out.push(Token { tok: Tok::Times, span: Span::new(from, off), implied: true });
                }
            }
            let mut text = String::new();
            let mut j = i;
            // After a vertical slash the digits are a superscript: an integer,
            // so a point after them is the end of a PRINT (`PRINT y|2.`).
            let after_bar = out.last().is_some_and(|t| t.tok == Tok::Bar);
            let mut seen_point = after_bar;
            while j < chars.len() {
                let d = chars[j].1;
                if d.is_ascii_digit() {
                    text.push(d);
                } else if d == '.' && !seen_point {
                    seen_point = true;
                    text.push(d);
                } else if d == '\u{7f}' {
                    // nullified: ignored even inside a number
                } else {
                    break;
                }
                j += 1;
            }
            out.push(Token { tok: Tok::Number(text), span: Span::new(off, end_of(j)), implied: false });
            i = j;
            continue;
        }
        // An exponent: ^ with an optional minus and digits, or superscripts.
        if c == '^' || c == '⁻' || super_digit(c).is_some() {
            let mut j = i;
            let mut neg = false;
            let mut digits = String::new();
            if c == '^' {
                j += 1;
                if chars.get(j).is_some_and(|&(_, d)| d == '-') {
                    neg = true;
                    j += 1;
                }
                while j < chars.len() && chars[j].1.is_ascii_digit() {
                    digits.push(chars[j].1);
                    j += 1;
                }
            } else {
                if c == '⁻' {
                    neg = true;
                    j += 1;
                }
                while j < chars.len() {
                    match super_digit(chars[j].1) {
                        Some(d) => digits.push(char::from_digit(d, 10).unwrap()),
                        None => break,
                    }
                    j += 1;
                }
            }
            if digits.is_empty() {
                out.push(Token { tok: Tok::Bad(c), span: Span::new(off, end_of(j.max(i + 1))), implied: false });
                i = j.max(i + 1);
                continue;
            }
            let v: i64 = digits.parse().unwrap_or(i64::MAX);
            out.push(Token {
                tok: Tok::Super(if neg { -v } else { v }),
                span: Span::new(off, end_of(j)),
                implied: false,
            });
            i = j;
            continue;
        }
        if c.is_ascii_uppercase() {
            let mut j = i;
            let mut w = String::new();
            while j < chars.len() && chars[j].1.is_ascii_uppercase() {
                w.push(chars[j].1);
                j += 1;
            }
            out.push(Token { tok: Tok::Word(w), span: Span::new(off, end_of(j)), implied: false });
            i = j;
            continue;
        }
        let tok = match c {
            'a'..='z' => Tok::Letter(c),
            '|' => Tok::Bar,
            '+' => Tok::Plus,
            '-' | '−' => Tok::Minus,
            '·' | ':' | '*' | '×' => Tok::Times,
            '/' => Tok::Slash,
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            ',' => Tok::Comma,
            '.' => Tok::Period,
            '=' => Tok::Equals,
            other => Tok::Bad(other),
        };
        out.push(Token { tok, span: Span::new(off, end_of(i + 1)), implied: false });
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(s: &str) -> Vec<Tok> {
        lex(s).into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn a_space_between_digits_is_times() {
        assert_eq!(toks("23"), vec![Tok::Number("23".into())]);
        assert_eq!(toks("2 3"), vec![Tok::Number("2".into()), Tok::Times, Tok::Number("3".into())]);
        assert_eq!(toks("2\n3"), vec![Tok::Number("2".into()), Tok::Times, Tok::Number("3".into())]);
        // between a letter and a digit a space is simply ignored
        assert_eq!(toks("a 3"), vec![Tok::Letter('a'), Tok::Number("3".into())]);
    }

    #[test]
    fn the_nullify_character_is_ignored_even_in_a_number() {
        assert_eq!(toks("2\u{7f}3"), vec![Tok::Number("23".into())]);
    }

    #[test]
    fn exponents_either_way() {
        assert_eq!(toks("x²"), toks("x^2"));
        assert_eq!(toks("a⁻³"), vec![Tok::Letter('a'), Tok::Super(-3)]);
        assert_eq!(toks("F^11(z)")[1], Tok::Super(11));
    }

    #[test]
    fn points() {
        assert_eq!(toks(".012"), vec![Tok::Number(".012".into())]);
        assert_eq!(toks("PRINT x."), vec![Tok::Word("PRINT".into()), Tok::Letter('x'), Tok::Period]);
        assert_eq!(toks("2. (0.25)")[0], Tok::Number("2.".into()));
        assert_eq!(toks("y|2.")[2..], [Tok::Number("2".into()), Tok::Period]);
    }
}
