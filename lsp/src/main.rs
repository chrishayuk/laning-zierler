//! A language server for the algebraic system of Laning and Zierler.
//!
//! It speaks the Language Server Protocol over standard input and output, so
//! any editor with an LSP client can use it. Everything it knows about a file
//! comes from the reconstruction itself: the lexer and parser read the tape,
//! the translator numbers the equations and makes their blocks of orders, and
//! `check` (the code behind `lz --check`) finds what is wrong. What the words
//! mean is in `src/info.rs`, in the words of E-364.
//!
//! The server reads one message at a time and keeps nothing but the open
//! documents, which lets it also be compiled for WASI and run inside an
//! editor or a web page with nothing to install (see `editors/vscode`).
//!
//! The protocol counts columns in UTF-16 code units; the tape has characters
//! like `·` and `²`. `Doc` converts between the two.

use laning_zierler::check::{check, Diagnostic, Severity};
use laning_zierler::info::{self, function_help, symbol_help, word_help, Help};
use laning_zierler::lex::{lex, superscript, Span, Tok, Token};
use laning_zierler::num::Num;
use laning_zierler::parse::{parse, FactorKind, Kind, Statement, Target, Var};
use laning_zierler::translate::{translate, Exec, Options, Program};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

const SEMANTIC_TYPES: &[&str] = &["comment", "number", "variable", "function", "keyword", "operator", "label", "parameter"];
const SEMANTIC_MODIFIERS: &[&str] = &["definition", "defaultLibrary", "readonly"];

/// What a token is, in its place on the tape.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    Note,
    /// The number in front of an equation.
    Label(u32),
    /// The number after SP, CP, SR or CR.
    JumpNumber(u32),
    /// An instruction: PRINT, STOP, SP, CP, SR, CR, ASSIGN, D, or N after a slash.
    Keyword,
    /// The F of a function subroutine.
    Function,
    /// The exponent after F: which function.
    FunctionNumber(u32),
    /// An exponent.
    Exponent(i64),
    /// A lower-case letter.
    Letter(char),
    /// The integer after a vertical slash: x|5.
    Index(u32),
    Number,
    Operator,
    Bad,
    /// The multiplication implied by a space between digits: no characters.
    Implied,
}

/// An open document and what was read out of it.
struct Doc {
    text: String,
    lines: Vec<Vec<char>>,
    line_starts: Vec<usize>,
    tokens: Vec<Token>,
    roles: Vec<Role>,
    statements: Vec<Statement>,
    program: Program,
    diagnostics: Vec<Diagnostic>,
}

impl Doc {
    fn new(text: &str) -> Doc {
        let tokens = lex(text);
        let tape = parse(text);
        let (program, _) = translate(text, &Options::default());
        let diagnostics = check(text, &Options::default());
        let mut line_starts = vec![0];
        for (i, c) in text.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }
        let lines = text.split('\n').map(|l| l.chars().collect()).collect();
        let roles = roles(&tokens, &tape.statements);
        Doc { text: text.to_string(), lines, line_starts, tokens, roles, statements: tape.statements, program, diagnostics }
    }

    fn has_d(&self) -> bool {
        self.program.d_count > 0
    }

    /// Line and column (in characters) of a byte offset.
    fn pos(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.text.len());
        let line = self.line_starts.partition_point(|&s| s <= offset) - 1;
        let col = self.text[self.line_starts[line]..offset].chars().count();
        (line, col)
    }

    /// The byte offset of a line and column.
    fn offset(&self, line: usize, col: usize) -> usize {
        let Some(&start) = self.line_starts.get(line) else { return self.text.len() };
        let chars = &self.lines[line];
        start + chars[..col.min(chars.len())].iter().map(|c| c.len_utf8()).sum::<usize>()
    }

    /// UTF-16 units before a column of a line.
    fn units(&self, line: usize, column: usize) -> usize {
        match self.lines.get(line) {
            Some(chars) => chars[..column.min(chars.len())].iter().map(|c| c.len_utf16()).sum(),
            None => 0,
        }
    }

    /// The column at a number of UTF-16 units into a line.
    fn column(&self, line: usize, mut units: usize) -> usize {
        let Some(chars) = self.lines.get(line) else { return 0 };
        for (column, c) in chars.iter().enumerate() {
            if units < c.len_utf16() {
                return column;
            }
            units -= c.len_utf16();
        }
        chars.len()
    }

    fn position(&self, offset: usize) -> Value {
        let (line, col) = self.pos(offset);
        json!({ "line": line, "character": self.units(line, col) })
    }

    fn range(&self, span: Span) -> Value {
        json!({ "start": self.position(span.start), "end": self.position(span.end) })
    }

    /// The byte offset of a position sent by the editor.
    fn place(&self, position: &Value) -> usize {
        let line = position["line"].as_u64().unwrap_or(0) as usize;
        let units = position["character"].as_u64().unwrap_or(0) as usize;
        self.offset(line, self.column(line, units))
    }

    /// The token at a position: the one under the cursor, or the one just
    /// before it.
    fn token_at(&self, position: &Value) -> Option<usize> {
        let off = self.place(position);
        let real = |i: &usize| self.roles[*i] != Role::Implied;
        (0..self.tokens.len())
            .filter(real)
            .find(|&i| self.tokens[i].span.start <= off && off < self.tokens[i].span.end)
            .or_else(|| (0..self.tokens.len()).filter(real).find(|&i| self.tokens[i].span.end == off))
    }

    fn source(&self, span: Span) -> &str {
        &self.text[span.start.min(self.text.len())..span.end.min(self.text.len())]
    }

    /// The statement (and its index) a byte offset falls in.
    fn statement_at(&self, off: usize) -> Option<usize> {
        self.statements.iter().position(|s| s.span.start <= off && off < s.span.end.max(s.span.start + 1))
    }

    /// The number the translator gives the i-th statement.
    fn number_of(&self, i: usize) -> Option<u32> {
        self.program.equations.get(i).and_then(|e| e.number)
    }

    fn labelled(&self, n: u32) -> Option<usize> {
        self.statements.iter().position(|s| s.label == Some(n))
    }

    /// Tokens that name equation n: its label and the jumps to it.
    fn equation_tokens(&self, n: u32) -> Vec<(usize, bool)> {
        (0..self.tokens.len())
            .filter_map(|i| match self.roles[i] {
                Role::Label(m) if m == n => Some((i, true)),
                Role::JumpNumber(m) if m == n => Some((i, false)),
                _ => None,
            })
            .collect()
    }

    /// Statements that give a value to a letter (plain, superscript or
    /// indexed), with the target's span.
    fn assignments(&self, c: char) -> Vec<(usize, Span)> {
        self.statements
            .iter()
            .enumerate()
            .filter_map(|(i, s)| match &s.kind {
                Kind::Equation { target, .. } | Kind::Derivative { target, .. } if target.var.letter() == c => Some((i, target.span)),
                Kind::AssignDrum { letter, .. } | Kind::List { letter, .. } if *letter == c => Some((i, s.span)),
                _ => None,
            })
            .collect()
    }

    /// Every token of a letter, and whether it is the left side of an equation.
    fn letter_tokens(&self, c: char) -> Vec<(usize, bool)> {
        let targets: Vec<usize> = self
            .statements
            .iter()
            .filter_map(|s| match &s.kind {
                Kind::Equation { target, .. } | Kind::Derivative { target, .. } => Some(target.span.start),
                _ => None,
            })
            .collect();
        (0..self.tokens.len())
            .filter(|&i| self.roles[i] == Role::Letter(c))
            .map(|i| (i, targets.contains(&self.tokens[i].span.start)))
            .collect()
    }
}

/// Give every token its role, from the statements around it.
fn roles(tokens: &[Token], statements: &[Statement]) -> Vec<Role> {
    let mut labels: BTreeMap<usize, u32> = BTreeMap::new();
    let mut jumps: BTreeMap<usize, u32> = BTreeMap::new();
    for s in statements {
        if let (Some(n), Some(sp)) = (s.label, s.label_span) {
            labels.insert(sp.start, n);
        }
        if let Kind::Jump { target: Target::Number(n), target_span, .. } = &s.kind {
            jumps.insert(target_span.start, *n);
        }
    }
    let mut out = Vec::with_capacity(tokens.len());
    let mut prev: Option<&Tok> = None;
    for t in tokens {
        let role = match &t.tok {
            Tok::Note => Role::Note,
            Tok::Times if t.implied => Role::Implied,
            Tok::Number(text) => {
                if let Some(&n) = labels.get(&t.span.start) {
                    Role::Label(n)
                } else if let Some(&n) = jumps.get(&t.span.start) {
                    Role::JumpNumber(n)
                } else if prev == Some(&Tok::Bar) {
                    Role::Index(text.parse().unwrap_or(0))
                } else {
                    Role::Number
                }
            }
            Tok::Super(n) => match prev {
                Some(Tok::Word(w)) if w == "F" => Role::FunctionNumber(*n as u32),
                Some(Tok::Bar) => Role::Index(*n as u32),
                _ => Role::Exponent(*n),
            },
            Tok::Word(w) => match w.as_str() {
                "F" => Role::Function,
                "PRINT" | "STOP" | "SP" | "CP" | "SR" | "CR" | "ASSIGN" | "D" | "N" => Role::Keyword,
                _ => Role::Bad,
            },
            Tok::Letter(c) => Role::Letter(*c),
            Tok::Bad(_) => Role::Bad,
            _ => Role::Operator,
        };
        out.push(role);
        if t.tok != Tok::Note {
            prev = Some(&t.tok);
        }
    }
    out
}

fn fenced(text: &str) -> String {
    format!("```laning-zierler\n{}\n```", text.trim_end())
}

/// A heading, a paragraph and an example, as the Markdown of a hover.
fn entry(title: &str, (heading, body, example): Help) -> String {
    format!("**`{title}`** {heading}\n\n{body}\n\n{}", fenced(example))
}

fn times(n: usize) -> String {
    match n {
        0 => "nowhere".into(),
        1 => "once".into(),
        2 => "twice".into(),
        n => format!("{n} times"),
    }
}

/// What equation n is, and where it is reached from.
fn describe_equation(doc: &Doc, n: u32) -> String {
    let Some(i) = doc.labelled(n) else {
        return format!("**Equation {n}**\n\nNo equation is numbered {n}.");
    };
    let s = &doc.statements[i];
    let mut text = format!("**Equation {n}**\n\n{}", fenced(doc.source(s.span)));
    let from: Vec<String> = doc
        .equation_tokens(n)
        .into_iter()
        .filter(|(_, defines)| !defines)
        .map(|(t, _)| {
            let (line, _) = doc.pos(doc.tokens[t].span.start);
            let word = doc.statement_at(doc.tokens[t].span.start).map(|k| match &doc.statements[k].kind {
                Kind::Jump { kind, .. } => kind.name(),
                _ => "",
            });
            format!("{} on line {}", word.unwrap_or(""), line + 1)
        })
        .collect();
    if from.is_empty() {
        text.push_str("\n\nNo SP, CP, SR or CR names it.");
    } else {
        text.push_str(&format!("\n\nReached by {}.", from.join(", ")));
    }
    text
}

fn describe_letter(doc: &Doc, c: char, as_index: bool) -> String {
    let mut text = format!("**`{c}`** a letter variable");
    if let Some(role) = info::letter_role(c, doc.has_d()) {
        text.push_str(&format!("\n\n`{c}` is {role}."));
    }
    let assigned = doc.assignments(c);
    let plain: Vec<String> = assigned
        .iter()
        .filter_map(|&(i, _)| match &doc.statements[i].kind {
            Kind::Equation { target, .. } | Kind::Derivative { target, .. } => {
                let (line, _) = doc.pos(doc.statements[i].span.start);
                let n = doc.number_of(i).map(|n| format!("equation {n}")).unwrap_or_default();
                Some(format!("`{}` in {n} (line {})", target.var, line + 1))
            }
            _ => None,
        })
        .collect();
    if plain.is_empty() {
        if !(doc.has_d() && (c == 't' || c == 'h')) {
            text.push_str(&format!("\n\nNo equation gives `{c}` a value: it reads as zero."));
        }
    } else {
        text.push_str(&format!("\n\nGiven a value {}: {}.", times(plain.len()), plain.join(", ")));
    }
    if let Some(g) = doc.program.drum.iter().find(|g| g.letter == c) {
        text.push_str(&format!("\n\n`{c}|1` to `{c}|{}` are on the drum.", g.values.len()));
    }
    if as_index {
        text.push_str(&format!("\n\nHere `{c}` is an index. {}", info::INDEX_RULE));
    }
    text
}

fn describe_superscript(doc: &Doc, c: char, n: u32) -> String {
    let mut text = format!("**`{c}|{n}`** a superscript variable, written `{c}|{}` on the Flexowriter", superscript(n as i64));
    if let Some(g) = doc.program.drum.iter().find(|g| g.letter == c) {
        if n >= 1 && (n as usize) <= g.values.len() {
            text.push_str(&format!(
                "\n\nOn the drum (one of `{c}|1` to `{c}|{}`), initially {}. Using it takes \"about 30 milliseconds longer\" than a variable in MS (p. 16).",
                g.values.len(),
                g.values[n as usize - 1]
            ));
            return text;
        }
    }
    text.push_str(&format!(
        "\n\nIn MS (core). There is room for {} superscript variables in this program: 250, less 2 for each D equation and 1 for each group on the drum.",
        doc.program.core_super_limit()
    ));
    text
}

/// The translation of the statement an `=` belongs to.
fn describe_translation(doc: &Doc, i: usize) -> Option<String> {
    let e = doc.program.equations.get(i)?;
    let (orders, last) = match &e.exec {
        Exec::Equation { orders, target } => (orders, format!("store {}", target.var)),
        Exec::Derivative { orders, target, slot } => (orders, format!("deriv d{}/dt -> slot {slot}", target.var)),
        _ => return None,
    };
    let number = match e.number {
        Some(n) if e.labelled => format!("Equation {n}"),
        Some(n) => format!("Equation {n} (numbered by the translator)"),
        None => "Equation".into(),
    };
    let mut block: Vec<String> = orders.iter().map(|o| o.to_string()).collect();
    block.push(last);
    Some(format!(
        "**{number}** translated into a block of {} interpretive orders, kept on the drum and brought into core when the equation is carried out:\n\n```text\n{}\n```",
        block.len(),
        block.join("\n")
    ))
}

fn hover(doc: &Doc, position: &Value) -> Value {
    let Some(i) = doc.token_at(position) else { return Value::Null };
    let t = &doc.tokens[i];
    let typed = doc.source(t.span).to_string();
    let prev = (0..i).rev().find(|&k| doc.roles[k] != Role::Note).map(|k| &doc.tokens[k].tok);
    let next = doc.tokens.get(i + 1).map(|t| &t.tok);
    let value = match doc.roles[i] {
        Role::Keyword => word_help(&typed).map(|h| entry(&typed, h)),
        Role::Function => {
            let mut s = entry("F", word_help("F").unwrap());
            if let Some(Tok::Super(k)) = next {
                if let Some((title, body)) = function_help(*k as u32) {
                    s = format!("**{title}**\n\n{body}\n\n---\n\n{s}");
                }
            }
            Some(s)
        }
        Role::FunctionNumber(k) => function_help(k).map(|(title, body)| format!("**{title}**\n\n{body}")),
        Role::Exponent(n) => symbol_help("^").map(|h| {
            format!("{}\n\nHere: to the power {n}.", entry(&typed, h))
        }),
        Role::Label(n) | Role::JumpNumber(n) => Some(describe_equation(doc, n)),
        Role::Letter(c) => {
            if next == Some(&Tok::Bar) {
                let sup = match doc.tokens.get(i + 2).map(|t| (&t.tok, doc.roles[i + 2])) {
                    Some((_, Role::Index(n))) => Some(describe_superscript(doc, c, n)),
                    Some((Tok::Word(w), _)) if w == "N" => word_help("N").map(|h| entry("N", h)),
                    _ => None,
                };
                Some(match sup {
                    Some(s) => format!("{s}\n\n---\n\n{}", describe_letter(doc, c, false)),
                    None => describe_letter(doc, c, false),
                })
            } else {
                let after_jump = matches!(prev, Some(Tok::Word(w)) if matches!(w.as_str(), "SP" | "CP" | "SR" | "CR"));
                let as_index = prev == Some(&Tok::Bar) || after_jump;
                Some(describe_letter(doc, c, as_index))
            }
        }
        Role::Index(n) => match (i >= 2).then(|| &doc.tokens[i - 2].tok) {
            Some(Tok::Letter(c)) => Some(describe_superscript(doc, *c, n)),
            _ => None,
        },
        Role::Number => match &t.tok {
            Tok::Number(text) => match laning_zierler::parse::number_value(text, t.span).ok().and_then(|v| Num::round(v).ok()) {
                Some(n) => Some(format!(
                    "**`{text}`** a number\n\nHeld in the 24, 6 system as `{n}`: exactly {}. \"It is useless to supply more than 8 significant decimal figures\" (p. 5).",
                    n.value()
                )),
                None => Some(format!("**`{text}`** a number too large for the 24, 6 system: read without its point it exceeds 2⁶³ - 2³⁹")),
            },
            _ => None,
        },
        Role::Operator => {
            let mut s = symbol_help(&typed).map(|h| entry(&typed, h));
            if typed == "=" {
                if let Some(k) = doc.statement_at(t.span.start) {
                    if let Some(tr) = describe_translation(doc, k) {
                        s = Some(format!("{tr}\n\n---\n\n{}", s.unwrap_or_default()));
                    }
                }
            }
            s
        }
        Role::Note => symbol_help("#").map(|h| entry("#", h)),
        Role::Bad | Role::Implied => None,
    };
    match value {
        Some(v) => json!({ "contents": { "kind": "markdown", "value": v }, "range": doc.range(t.span) }),
        None => Value::Null,
    }
}

fn location(doc: &Doc, uri: &str, span: Span) -> Value {
    json!({ "uri": uri, "range": doc.range(span) })
}

fn definition(doc: &Doc, uri: &str, position: &Value) -> Value {
    let Some(i) = doc.token_at(position) else { return Value::Null };
    match doc.roles[i] {
        Role::Label(n) | Role::JumpNumber(n) => match doc.labelled(n) {
            Some(k) => location(doc, uri, doc.statements[k].label_span.unwrap()),
            None => Value::Null,
        },
        Role::Letter(c) => {
            // x|5 goes to where x|5 is given a value; a letter to its first equation.
            let sup = match doc.roles.get(i + 2) {
                Some(Role::Index(n)) if doc.tokens.get(i + 1).map(|t| &t.tok) == Some(&Tok::Bar) => Some(*n),
                _ => None,
            };
            let found = doc.assignments(c).into_iter().find(|&(k, _)| match (&doc.statements[k].kind, sup) {
                (Kind::Equation { target, .. } | Kind::Derivative { target, .. }, Some(n)) => target.var == Var::Super(c, n),
                (Kind::AssignDrum { count, .. }, Some(n)) => n >= 1 && n <= *count,
                (Kind::List { .. }, Some(_)) => true,
                (Kind::Equation { target, .. } | Kind::Derivative { target, .. }, None) => target.var == Var::Letter(c),
                _ => false,
            });
            match found {
                Some((_, span)) => location(doc, uri, span),
                None => Value::Null,
            }
        }
        _ => Value::Null,
    }
}

/// Tokens that mean the same thing as the one at the position, and whether
/// each defines it.
fn related(doc: &Doc, position: &Value) -> Vec<(usize, bool)> {
    let Some(i) = doc.token_at(position) else { return vec![] };
    match doc.roles[i] {
        Role::Label(n) | Role::JumpNumber(n) => doc.equation_tokens(n),
        Role::Letter(c) => doc.letter_tokens(c),
        _ => vec![],
    }
}

fn prepare_rename(doc: &Doc, position: &Value) -> Value {
    let Some(i) = doc.token_at(position) else { return Value::Null };
    match doc.roles[i] {
        Role::Label(_) | Role::JumpNumber(_) | Role::Letter(_) => doc.range(doc.tokens[i].span),
        _ => Value::Null,
    }
}

fn rename(doc: &Doc, uri: &str, position: &Value, new_name: &str) -> Result<Value, String> {
    let Some(i) = doc.token_at(position) else { return Err("Only an equation number or a letter can be renamed.".into()) };
    let new_name = new_name.trim();
    let tokens = match doc.roles[i] {
        Role::Label(n) | Role::JumpNumber(n) => {
            let m: u32 = new_name.parse().map_err(|_| "An equation number is a whole number from 1 to 100.".to_string())?;
            if !(1..=100).contains(&m) {
                return Err("Equations are numbered 1 to 100; 101 to 200 are the translator's.".into());
            }
            if m != n && doc.labelled(m).is_some() {
                return Err(format!("An equation is already numbered {m}."));
            }
            doc.equation_tokens(n)
        }
        Role::Letter(c) => {
            let mut chars = new_name.chars();
            let (Some(d), None) = (chars.next(), chars.next()) else {
                return Err("A variable is one lower-case letter.".into());
            };
            if !d.is_ascii_lowercase() {
                return Err("A variable is one lower-case letter.".into());
            }
            if doc.has_d() && (c == 't' || c == 'h' || d == 't' || d == 'h') {
                return Err("With D equations, t is the independent variable and h its increment: they keep their letters.".into());
            }
            if d != c && doc.roles.contains(&Role::Letter(d)) {
                return Err(format!("{d} is already in use."));
            }
            doc.letter_tokens(c)
        }
        _ => return Err("Only an equation number or a letter can be renamed.".into()),
    };
    let edits: Vec<Value> = tokens.iter().map(|&(t, _)| json!({ "range": doc.range(doc.tokens[t].span), "newText": new_name })).collect();
    Ok(json!({ "changes": { uri: edits } }))
}

fn item(label: &str, kind: u32, detail: &str, documentation: &str, order: &str) -> Value {
    json!({
        "label": label,
        "kind": kind,
        "detail": detail,
        "documentation": { "kind": "markdown", "value": documentation },
        "sortText": order,
    })
}

/// Templates offered at the start of a statement.
const TEMPLATES: &[(&str, &str, &str)] = &[
    ("counted loop", "n = 0, ... e = n - 9.5, CP 1,", "n = 0,\n1   ${1:x = x + 1},\n    n = n + 1,\n    e = n - ${2:9}.5,\n    CP 1,\n"),
    ("differential equations", "t, h, D y|1 = ..., stepping to a limit", "t = ${1:0},\nh = ${2:0.1},\n2   D ${3:y} = ${4:-y},\n    e = t - ${5:0.95},\n    CP 2,\n"),
    ("table on the drum", "a|N = 1 (1) 10,", "${1:a}|N = ${2:1} (${3:1}) ${4:10},"),
];

fn completion(doc: &Doc, position: &Value) -> Value {
    let off = doc.place(position);
    // The last real token before the cursor.
    let before: Vec<usize> =
        (0..doc.tokens.len()).filter(|&i| doc.tokens[i].span.end <= off && !matches!(doc.roles[i], Role::Note | Role::Implied)).collect();
    let prev = before.last().copied();
    let prev_tok = prev.map(|i| &doc.tokens[i].tok);
    let mut items = Vec::new();
    let letters: Vec<char> = {
        let mut v: Vec<char> = doc.roles.iter().filter_map(|r| if let Role::Letter(c) = r { Some(*c) } else { None }).collect();
        v.sort();
        v.dedup();
        v
    };
    let letter_items = |items: &mut Vec<Value>| {
        for &c in &letters {
            let assigned = doc.assignments(c).len();
            items.push(item(&c.to_string(), 6, &format!("given a value {}", times(assigned)), &describe_letter(doc, c, false), &format!("1{c}")));
        }
    };
    let function_items = |items: &mut Vec<Value>, after_f: bool| {
        for k in 1..=23u32 {
            let (title, body) = function_help(k).unwrap();
            let mut it = item(&format!("F^{k}"), 3, &title, &body, &format!("2{k:02}"));
            it["filterText"] = json!(if after_f { format!("^{k}") } else { format!("F^{k}") });
            it["insertTextFormat"] = json!(2);
            it["insertText"] = json!(if after_f { format!("^{k}($1)") } else { format!("F^{k}($1)") });
            items.push(it);
        }
    };
    match prev_tok {
        Some(Tok::Word(w)) if matches!(w.as_str(), "SP" | "CP" | "SR" | "CR") => {
            for (k, s) in doc.statements.iter().enumerate() {
                if let Some(n) = s.label {
                    let text: String = doc.source(s.span).split_whitespace().collect::<Vec<_>>().join(" ");
                    items.push(item(&n.to_string(), 12, &text, &describe_equation(doc, n), &format!("0{n:03}{k}")));
                }
            }
            letter_items(&mut items);
        }
        Some(Tok::Word(w)) if w == "F" => function_items(&mut items, true),
        Some(Tok::Bar) => {
            items.push(item("N", 14, "values for a whole table, on the drum", &entry("N", word_help("N").unwrap()), "0"));
            letter_items(&mut items);
        }
        _ => {
            let at_start = match prev {
                None => true,
                Some(p) => matches!(doc.tokens[p].tok, Tok::Comma | Tok::Period) || matches!(doc.roles[p], Role::Label(_)),
            };
            if at_start {
                for w in info::WORDS.iter().filter(|w| !matches!(**w, "F" | "N")) {
                    let (heading, _, _) = word_help(w).unwrap();
                    let mut it = item(w, 14, heading, &entry(w, word_help(w).unwrap()), &format!("0{w}"));
                    let snippet = match *w {
                        "PRINT" => "PRINT ${1:x}.",
                        "SP" | "CP" | "SR" | "CR" => "",
                        "ASSIGN" => "ASSIGN ${1:a}|${2:100},",
                        "D" => "D ${1:y} = $0,",
                        _ => "",
                    };
                    let snippet = if snippet.is_empty() && matches!(*w, "SP" | "CP" | "SR" | "CR") { format!("{w} ${{1:1}},") } else { snippet.to_string() };
                    if !snippet.is_empty() {
                        it["insertText"] = json!(snippet);
                        it["insertTextFormat"] = json!(2);
                    }
                    items.push(it);
                }
                for (label, detail, body) in TEMPLATES {
                    let mut it = item(label, 15, detail, &fenced(&body.replace("${", "").replace('}', "")), &format!("3{label}"));
                    it["insertText"] = json!(body);
                    it["insertTextFormat"] = json!(2);
                    items.push(it);
                }
            }
            letter_items(&mut items);
            if !at_start {
                function_items(&mut items, false);
            }
        }
    }
    json!({ "isIncomplete": false, "items": items })
}

/// Inside the parentheses of F^k( ... ): which function.
fn signature_help(doc: &Doc, position: &Value) -> Value {
    let off = doc.place(position);
    let mut depth = 0i32;
    let mut i = doc.tokens.iter().rposition(|t| t.span.end <= off);
    while let Some(k) = i {
        match doc.tokens[k].tok {
            Tok::Comma | Tok::Period => break,
            Tok::RParen => depth += 1,
            Tok::LParen => {
                if depth == 0 {
                    if k >= 2 {
                        if let (Tok::Word(w), Tok::Super(n)) = (&doc.tokens[k - 2].tok, &doc.tokens[k - 1].tok) {
                            if w == "F" {
                                if let Some((title, body)) = function_help(*n as u32) {
                                    let label = format!("F{}(x)", superscript(*n));
                                    return json!({
                                        "signatures": [{ "label": label, "documentation": { "kind": "markdown", "value": format!("**{title}**\n\n{body}") },
                                            "parameters": [{ "label": "x" }] }],
                                        "activeSignature": 0, "activeParameter": 0,
                                    });
                                }
                            }
                        }
                    }
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
        i = k.checked_sub(1);
    }
    Value::Null
}

fn symbols(doc: &Doc) -> Value {
    let mut out = Vec::new();
    for s in &doc.statements {
        let text: String = doc.source(s.span).split_whitespace().collect::<Vec<_>>().join(" ");
        let (name, kind) = match (&s.kind, s.label) {
            (Kind::Derivative { target, .. }, l) => (format!("{}D{}", l.map(|n| format!("{n}  ")).unwrap_or_default(), target.var), 12),
            (Kind::AssignDrum { letter, count }, _) => (format!("{letter}|1 to {letter}|{count} on the drum"), 18),
            (Kind::List { letter, .. }, _) => (format!("{letter}|N on the drum"), 18),
            (Kind::Stop, Some(n)) => (format!("{n}  STOP"), 20),
            (_, Some(n)) => (format!("Equation {n}"), 12),
            _ => continue,
        };
        out.push(json!({
            "name": name,
            "detail": text,
            "kind": kind,
            "range": doc.range(s.span),
            "selectionRange": doc.range(s.label_span.unwrap_or(s.span)),
        }));
    }
    json!(out)
}

/// The translator's numbers for the equations the programmer did not number.
fn inlay_hints(doc: &Doc) -> Value {
    let mut out = Vec::new();
    for (i, s) in doc.statements.iter().enumerate() {
        if s.label.is_some() {
            continue;
        }
        if let Some(n) = doc.number_of(i) {
            out.push(json!({
                "position": doc.position(s.span.start),
                "label": format!("{n}"),
                "paddingRight": true,
                "tooltip": "Equations not numbered by the programmer \"are automatically assigned numbers from 101 to 200 by the computer in the order in which they are written\" (E-364 p. 20). An alarm names equations by these numbers.",
            }));
        }
    }
    json!(out)
}

fn folding(doc: &Doc) -> Value {
    let mut out = Vec::new();
    if let (Some(f), Some(l)) = (doc.program.first_d, doc.program.last_d) {
        let (a, _) = doc.pos(doc.statements[f].span.start);
        let (b, _) = doc.pos(doc.statements[l].span.end);
        if b > a {
            out.push(json!({ "startLine": a, "endLine": b, "kind": "region" }));
        }
    }
    // Runs of lines that are only notes.
    let note_line = |l: usize| doc.lines[l].iter().collect::<String>().trim_start().starts_with('#');
    let mut l = 0;
    while l < doc.lines.len() {
        if note_line(l) {
            let start = l;
            while l + 1 < doc.lines.len() && note_line(l + 1) {
                l += 1;
            }
            if l > start {
                out.push(json!({ "startLine": start, "endLine": l, "kind": "comment" }));
            }
        }
        l += 1;
    }
    json!(out)
}

fn semantic(doc: &Doc, i: usize) -> Option<(&'static str, u32)> {
    let def = 1u32;
    let lib = 2u32;
    let ro = 4u32;
    Some(match doc.roles[i] {
        Role::Note => ("comment", 0),
        Role::Label(_) => ("label", def),
        Role::JumpNumber(_) => ("label", 0),
        Role::Keyword => ("keyword", 0),
        Role::Function | Role::FunctionNumber(_) => ("function", lib),
        Role::Exponent(_) => ("operator", 0),
        Role::Letter(c) => {
            let defines = doc.letter_tokens(c).iter().any(|&(t, d)| t == i && d);
            let special = doc.has_d() && (c == 't' || c == 'h');
            ("variable", if defines { def } else { 0 } | if special { lib | ro } else { 0 })
        }
        Role::Index(_) => ("parameter", 0),
        Role::Number => ("number", 0),
        Role::Operator => ("operator", 0),
        Role::Bad | Role::Implied => return None,
    })
}

fn semantic_tokens(doc: &Doc) -> Vec<u32> {
    let mut data = Vec::new();
    let (mut last_line, mut last_start) = (0usize, 0usize);
    for i in 0..doc.tokens.len() {
        let Some((kind, mods)) = semantic(doc, i) else { continue };
        let t = &doc.tokens[i];
        let (line, col) = doc.pos(t.span.start);
        let (end_line, end_col) = doc.pos(t.span.end);
        if end_line != line {
            continue;
        }
        let start = doc.units(line, col);
        let length = doc.units(line, end_col) - start;
        let delta_start = if line == last_line { start - last_start } else { start };
        data.extend([(line - last_line) as u32, delta_start as u32, length as u32, SEMANTIC_TYPES.iter().position(|k| *k == kind).unwrap() as u32, mods]);
        last_line = line;
        last_start = start;
    }
    data
}

/// Edits that write the Flexowriter's characters for the stand-ins: `²` for
/// `^2`, `⁻³` for `^-3`, `·` for `*`.
fn flexowriter_characters(doc: &Doc, range: Option<(usize, usize)>) -> Vec<Value> {
    let mut out = Vec::new();
    for t in &doc.tokens {
        if let Some((a, b)) = range {
            if t.span.end < a || t.span.start > b {
                continue;
            }
        }
        let typed = doc.source(t.span);
        let real = match &t.tok {
            Tok::Super(n) if typed.starts_with('^') => superscript(*n),
            Tok::Times if typed == "*" => "·".to_string(),
            _ => continue,
        };
        out.push(json!({ "range": doc.range(t.span), "newText": real }));
    }
    out
}

/// The fix for the CP hint: subtract a half, as the manual's loops do.
fn half_fix(doc: &Doc, uri: &str, d: &Diagnostic) -> Option<Value> {
    let k = doc.statements.iter().position(|s| s.span.start == d.span.start)?;
    let prev = doc.statements[..k].iter().rev().find(|s| matches!(s.kind, Kind::Equation { .. } | Kind::Derivative { .. }))?;
    let Kind::Equation { expr, .. } = &prev.kind else { return None };
    if expr.terms.len() != 2 || !expr.terms[1].0 {
        return None;
    }
    let constant = |t: &laning_zierler::parse::Term| match t.factors.as_slice() {
        [(false, f)] if f.power.is_none() => match &f.kind {
            FactorKind::Number { value, .. } if value.fract() == 0.0 => Some((*value, f.span)),
            _ => None,
        },
        _ => None,
    };
    // x - 10 becomes x - 9.5; 10 - x becomes 10.5 - x.
    let (new, span) = if let Some((v, span)) = constant(&expr.terms[1].1) {
        (v - 0.5, span)
    } else {
        let (v, span) = constant(&expr.terms[0].1)?;
        (v + 0.5, span)
    };
    let text = format!("{new}");
    Some(json!({
        "title": format!("Write {text}, so that the difference is never zero"),
        "kind": "quickfix",
        "diagnostics": [diagnostic_json(doc, d)],
        "isPreferred": true,
        "edit": { "changes": { uri: [{ "range": doc.range(span), "newText": text }] } },
    }))
}

fn code_actions(doc: &Doc, uri: &str, range: &Value) -> Value {
    let a = doc.place(&range["start"]);
    let b = doc.place(&range["end"]);
    let mut actions = Vec::new();
    for d in &doc.diagnostics {
        if d.severity == Severity::Hint && d.message.contains("may or may not") && d.span.start <= b && a <= d.span.end.max(d.span.start) {
            if let Some(fix) = half_fix(doc, uri, d) {
                actions.push(fix);
            }
        }
    }
    let here = flexowriter_characters(doc, Some((a, b)));
    if !here.is_empty() {
        actions.push(json!({ "title": "Write the Flexowriter's characters here (² for ^2, · for *)", "kind": "quickfix", "edit": { "changes": { uri: here } } }));
    }
    let all = flexowriter_characters(doc, None);
    if !all.is_empty() {
        actions.push(json!({ "title": "Write the Flexowriter's characters for every stand-in", "kind": "source.fixAll", "edit": { "changes": { uri: all } } }));
    }
    json!(actions)
}

fn diagnostic_json(doc: &Doc, d: &Diagnostic) -> Value {
    let mut span = d.span;
    if span.end <= span.start {
        // An empty span (the end of the tape): mark one character, or none.
        span.end = (span.start + 1).min(doc.text.len());
        if span.end < span.start {
            span.end = span.start;
        }
        while span.end < doc.text.len() && !doc.text.is_char_boundary(span.end) {
            span.end += 1;
        }
    }
    json!({
        "range": doc.range(span),
        "severity": match d.severity { Severity::Error => 1, Severity::Warning => 2, Severity::Hint => 4 },
        "source": "lz",
        "message": d.message,
    })
}

struct Server {
    docs: BTreeMap<String, Doc>,
    shutdown: bool,
    exit: bool,
}

impl Server {
    fn new() -> Self {
        Server { docs: BTreeMap::new(), shutdown: false, exit: false }
    }

    /// Handles one incoming message and returns the messages to send back.
    fn handle(&mut self, message: &Value) -> Vec<Value> {
        let method = message["method"].as_str().unwrap_or("");
        let params = &message["params"];
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();

        let Some(id) = message.get("id").filter(|_| !method.is_empty()) else {
            return match method {
                "textDocument/didOpen" => self.open(uri, params["textDocument"]["text"].as_str().unwrap_or("")),
                "textDocument/didChange" => {
                    // Full sync: the last change holds the whole text.
                    let changes = params["contentChanges"].as_array();
                    match changes.and_then(|c| c.last()).and_then(|c| c["text"].as_str()) {
                        Some(text) => self.open(uri, text),
                        None => vec![],
                    }
                }
                "textDocument/didClose" => {
                    self.docs.remove(&uri);
                    vec![json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                        "params": { "uri": uri, "diagnostics": [] } })]
                }
                "exit" => {
                    self.exit = true;
                    vec![]
                }
                _ => vec![],
            };
        };

        let reply = |result: Value| vec![json!({ "jsonrpc": "2.0", "id": id, "result": result })];
        let fail = |code: i32, text: &str| vec![json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": text } })];

        match method {
            "initialize" => return reply(self.capabilities()),
            "shutdown" => {
                self.shutdown = true;
                return reply(Value::Null);
            }
            _ => {}
        }
        if !method.starts_with("textDocument/") {
            return fail(-32601, &format!("method not supported: {method}"));
        }
        let Some(doc) = self.docs.get(&uri) else {
            return reply(Value::Null);
        };
        let position = &params["position"];

        match method {
            "textDocument/hover" => reply(hover(doc, position)),
            "textDocument/definition" => reply(definition(doc, &uri, position)),
            "textDocument/references" => {
                let with_declaration = params["context"]["includeDeclaration"].as_bool().unwrap_or(true);
                let found: Vec<Value> = related(doc, position)
                    .into_iter()
                    .filter(|(_, defines)| with_declaration || !defines)
                    .map(|(t, _)| location(doc, &uri, doc.tokens[t].span))
                    .collect();
                reply(json!(found))
            }
            "textDocument/documentHighlight" => {
                let found: Vec<Value> = related(doc, position)
                    .into_iter()
                    .map(|(t, defines)| json!({ "range": doc.range(doc.tokens[t].span), "kind": if defines { 3 } else { 2 } }))
                    .collect();
                reply(json!(found))
            }
            "textDocument/prepareRename" => reply(prepare_rename(doc, position)),
            "textDocument/rename" => match rename(doc, &uri, position, params["newName"].as_str().unwrap_or("")) {
                Ok(edit) => reply(edit),
                Err(text) => fail(-32602, &text),
            },
            "textDocument/completion" => reply(completion(doc, position)),
            "textDocument/signatureHelp" => reply(signature_help(doc, position)),
            "textDocument/documentSymbol" => reply(symbols(doc)),
            "textDocument/foldingRange" => reply(folding(doc)),
            "textDocument/inlayHint" => reply(inlay_hints(doc)),
            "textDocument/semanticTokens/full" => reply(json!({ "data": semantic_tokens(doc) })),
            "textDocument/formatting" => reply(json!(flexowriter_characters(doc, None))),
            "textDocument/codeAction" => reply(code_actions(doc, &uri, &params["range"])),
            _ => fail(-32601, &format!("method not supported: {method}")),
        }
    }

    fn capabilities(&self) -> Value {
        json!({
            "serverInfo": { "name": "lz-lsp", "version": env!("CARGO_PKG_VERSION") },
            "capabilities": {
                "positionEncoding": "utf-16",
                "textDocumentSync": { "openClose": true, "change": 1 },
                "hoverProvider": true,
                "definitionProvider": true,
                "referencesProvider": true,
                "documentHighlightProvider": true,
                "renameProvider": { "prepareProvider": true },
                "completionProvider": { "triggerCharacters": ["F", "^", "|", " "] },
                "signatureHelpProvider": { "triggerCharacters": ["("] },
                "documentSymbolProvider": true,
                "foldingRangeProvider": true,
                "inlayHintProvider": true,
                "documentFormattingProvider": true,
                "codeActionProvider": { "codeActionKinds": ["quickfix", "source.fixAll"] },
                "semanticTokensProvider": {
                    "legend": { "tokenTypes": SEMANTIC_TYPES, "tokenModifiers": SEMANTIC_MODIFIERS },
                    "full": true,
                },
            },
        })
    }

    fn open(&mut self, uri: String, text: &str) -> Vec<Value> {
        let doc = Doc::new(text);
        let diagnostics: Vec<Value> = doc.diagnostics.iter().map(|d| diagnostic_json(&doc, d)).collect();
        self.docs.insert(uri.clone(), doc);
        vec![json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
            "params": { "uri": uri, "diagnostics": diagnostics } })]
    }
}

fn read_message(input: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let line = line.trim_end();
        if line.is_empty() {
            if length.is_some() {
                break;
            }
            continue;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let mut body = vec![0; length.unwrap()];
    input.read_exact(&mut body)?;
    serde_json::from_slice(&body).map(Some).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn main() -> io::Result<()> {
    if std::env::args().any(|arg| arg == "--version") {
        println!("lz-lsp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut output = io::stdout().lock();
    let mut server = Server::new();
    while let Some(message) = read_message(&mut input)? {
        for reply in server.handle(&message) {
            let body = reply.to_string();
            write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
        }
        output.flush()?;
        if server.exit {
            break;
        }
    }
    std::process::exit(if server.shutdown { 0 } else { 1 });
}

#[cfg(test)]
mod tests;
