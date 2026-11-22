//! Running a translated program: "a computer within a computer".
//!
//! Equations are carried out in the order in which they are written, except
//! where SP, CP, SR or CR say otherwise. The answers are typed: the page is
//! what the delayed printer would have printed.

use std::collections::HashMap;

use crate::num::{function, format, Alarm, Num, ZeroModel};
use crate::parse::{Target, Var, VarRef};
use crate::translate::{Equation, Exec, Order, Program};

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub zero: ZeroModel,
    /// Give up after this many equations have been carried out.
    pub limit: u64,
    /// Keep a trace of every equation carried out.
    pub trace: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        RunOptions { zero: ZeroModel::default(), limit: 5_000_000, trace: false }
    }
}

/// What the run cost, counted: the raw material for the question of how slow
/// the system was. Only the drum variable's "about 30 milliseconds" is a figure
/// from 1954; see the README.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stats {
    /// Equations (and PRINTs, jumps, STOP) carried out.
    pub statements: u64,
    /// Blocks brought from the drum to core: one for each equation carried
    /// out that is not the one already in core.
    pub block_transfers: u64,
    /// Interpretive orders obeyed.
    pub orders: u64,
    /// Floating-point additions, subtractions, multiplications, divisions.
    pub arithmetic: u64,
    pub function_calls: u64,
    /// Reads and writes of variables on the drum.
    pub drum_variable_accesses: u64,
    /// Steps of the differential equations (each is four passes).
    pub integration_steps: u64,
    pub printed_values: u64,
}

/// What stopped a program other than STOP.
#[derive(Clone, Debug, PartialEq)]
pub struct AlarmReport {
    /// The number of the equation in which the alarm occurred.
    pub equation: Option<u32>,
    /// The number of the equation before it.
    pub preceding: Option<u32>,
    /// Why, in words. This is not what Whirlwind typed: it typed the numbers.
    pub reason: String,
    /// Index of the equation.
    pub index: usize,
}

#[derive(Clone, Debug)]
pub struct Outcome {
    /// The typed page.
    pub page: String,
    pub alarm: Option<AlarmReport>,
    /// The run was cut off by the limit.
    pub limit_reached: bool,
    pub stats: Stats,
    pub trace: Vec<String>,
}

/// Gill's variation of the fourth-order Runge-Kutta method, in the form
/// r = a(k - bq), y = y + r, q = q + 3r - ck, with k = h·f.
const GILL_A: [f64; 4] = [0.5, 1.0 - std::f64::consts::FRAC_1_SQRT_2, 1.0 + std::f64::consts::FRAC_1_SQRT_2, 1.0 / 6.0];
const GILL_B: [f64; 4] = [2.0, 1.0, 1.0, 2.0];
const GILL_C: [f64; 4] = [0.5, 1.0 - std::f64::consts::FRAC_1_SQRT_2, 1.0 + std::f64::consts::FRAC_1_SQRT_2, 0.5];

struct Machine<'a> {
    program: &'a Program,
    opts: &'a RunOptions,
    letters: [Num; 26],
    /// Superscript variables in core, in the order they were first stored.
    core: HashMap<(char, u32), Num>,
    /// Groups on the drum: letter to values (index 1 is element 0).
    drum: HashMap<char, Vec<Num>>,
    /// The last computed quantity, which CP and CR test.
    last: Num,
    /// Differential equations: k = h·f and Gill's q, by slot.
    k: Vec<Num>,
    q: Vec<Num>,
    stage: usize,
    t0: Num,
    in_core: Option<usize>,
    stats: Stats,
    page: String,
    column: usize,
    trace: Vec<String>,
}

type R<T> = Result<T, String>;

fn alarm(e: Alarm) -> String {
    e.to_string()
}

pub fn run(program: &Program, opts: &RunOptions) -> Outcome {
    let mut m = Machine {
        program,
        opts,
        letters: [Num::ZERO; 26],
        core: HashMap::new(),
        drum: program.drum.iter().map(|g| (g.letter, g.values.clone())).collect(),
        last: Num::ZERO,
        k: vec![Num::ZERO; program.d_count],
        q: vec![Num::ZERO; program.d_count],
        stage: 0,
        t0: Num::ZERO,
        in_core: None,
        stats: Stats::default(),
        page: String::new(),
        column: 0,
        trace: Vec::new(),
    };
    let mut pc = 0usize;
    let mut preceding: Option<u32> = None;
    // After SR or CR: where to come back to once the one equation is done.
    let mut come_back: Option<usize> = None;
    let mut alarm_report = None;
    let mut limit_reached = false;
    while pc < program.equations.len() {
        if m.stats.statements >= opts.limit {
            limit_reached = true;
            break;
        }
        m.stats.statements += 1;
        let eq = &program.equations[pc];
        match m.step(pc, eq) {
            Ok(Flow::Next) => pc += 1,
            Ok(Flow::Goto(i)) => pc = i,
            Ok(Flow::Call(i)) => {
                come_back = Some(pc + 1);
                pc = i;
                continue;
            }
            Ok(Flow::Stop) => break,
            Err(reason) => {
                alarm_report = Some(AlarmReport { equation: eq.number, preceding, reason, index: pc });
                break;
            }
        }
        // SR: "equation 5 to be executed next and then, after equation 5, the
        // equation or instruction following the SR 5".
        if let Some(back) = come_back.take() {
            pc = back;
        }
        if eq.number.is_some() {
            preceding = eq.number;
        }
    }
    if let Some(report) = &alarm_report {
        m.type_alarm(report);
        // "The programmer may obtain the values of any variables he desires
        // after an arithmetic alarm by writing the appropriate PRINT
        // instruction as equation 100."
        if let Some(Equation { exec: Exec::Print(items), .. }) = program.by_number(100) {
            let items = items.clone();
            if let Err(e) = m.print(&items) {
                m.page.push_str(&format!("\n{e}\n"));
            }
        }
    }
    Outcome { page: m.page, alarm: alarm_report, limit_reached, stats: m.stats, trace: m.trace }
}

enum Flow {
    Next,
    Goto(usize),
    /// SR or CR taken: carry out this one equation, then come back.
    Call(usize),
    Stop,
}

impl<'a> Machine<'a> {
    fn step(&mut self, pc: usize, eq: &Equation) -> R<Flow> {
        let z = self.opts.zero;
        match &eq.exec {
            Exec::Equation { orders, target } => {
                self.bring_in(pc);
                let v = self.eval(orders)?;
                self.store(&target.var, v)?;
                self.last = v;
                if self.opts.trace {
                    self.trace.push(format!("{:>3}  {} = {}", eq.number.unwrap_or(0), self.name(&target.var)?, format(v)));
                }
                Ok(Flow::Next)
            }
            Exec::Derivative { orders, target, slot } => {
                self.bring_in(pc);
                let f = self.eval(orders)?;
                self.last = f;
                let h = self.letters[idx('h')];
                self.k[*slot] = f.mul(h, z).map_err(alarm)?;
                self.stats.arithmetic += 1;
                if self.opts.trace {
                    self.trace.push(format!("{:>3}  D{} = {}   (pass {})", eq.number.unwrap_or(0), self.name(&target.var)?, format(f), self.stage + 1));
                }
                if Some(pc) == self.program.last_d {
                    return self.gill();
                }
                Ok(Flow::Next)
            }
            Exec::Print(items) => {
                self.print(items)?;
                Ok(Flow::Next)
            }
            Exec::Jump { kind, target } => {
                let taken = !kind.conditional() || self.last.is_negative();
                if self.opts.trace {
                    let t = match target {
                        Target::Number(n) => n.to_string(),
                        Target::Letter(c) => c.to_string(),
                    };
                    let what = if taken { "taken" } else { "not taken" };
                    self.trace.push(format!("     {} {t}   {what}", kind.name()));
                }
                if !taken {
                    return Ok(Flow::Next);
                }
                let n = match target {
                    Target::Number(n) => *n,
                    Target::Letter(c) => self.index_value(*c, "the instruction's equation number")?,
                };
                let Some(&i) = self.program.labels.get(&n) else {
                    return Err(format!("{} {n}: there is no equation numbered {n}", kind.name()));
                };
                Ok(if kind.returns() { Flow::Call(i) } else { Flow::Goto(i) })
            }
            Exec::Declaration => Ok(Flow::Next),
            Exec::Stop => Ok(Flow::Stop),
        }
    }

    fn bring_in(&mut self, pc: usize) {
        if self.in_core != Some(pc) {
            self.stats.block_transfers += 1;
            self.in_core = Some(pc);
        }
    }

    /// The end of a pass through the differential equations: move every
    /// dependent variable on by Gill's formulas, then either go back to the
    /// first D equation for the next pass or, after the fourth, go on.
    fn gill(&mut self) -> R<Flow> {
        let z = self.opts.zero;
        let j = self.stage;
        let a = Num::from_f64(GILL_A[j]);
        let b = Num::from_f64(GILL_B[j]);
        let c = Num::from_f64(GILL_C[j]);
        let three = Num::from_f64(3.0);
        if j == 0 {
            self.t0 = self.letters[idx('t')];
        }
        let targets: Vec<(usize, VarRef)> = self
            .program
            .equations
            .iter()
            .filter_map(|e| match &e.exec {
                Exec::Derivative { target, slot, .. } => Some((*slot, target.clone())),
                _ => None,
            })
            .collect();
        for (slot, target) in targets {
            let k = self.k[slot];
            let q = self.q[slot];
            // r = a(k - bq)
            let r = a.mul(k.sub(b.mul(q, z).map_err(alarm)?, z).map_err(alarm)?, z).map_err(alarm)?;
            let y = self.load(&target.var)?;
            self.store(&target.var, y.add(r, z).map_err(alarm)?)?;
            // q = q + 3r - ck
            let q = q.add(three.mul(r, z).map_err(alarm)?, z).map_err(alarm)?.sub(c.mul(k, z).map_err(alarm)?, z).map_err(alarm)?;
            self.q[slot] = q;
            self.stats.arithmetic += 8;
        }
        let h = self.letters[idx('h')];
        match j {
            0 => {
                let half = h.mul(Num::from_f64(0.5), z).map_err(alarm)?;
                self.letters[idx('t')] = self.t0.add(half, z).map_err(alarm)?;
            }
            2 => {
                self.letters[idx('t')] = self.t0.add(h, z).map_err(alarm)?;
            }
            _ => {}
        }
        if j < 3 {
            self.stage += 1;
            Ok(Flow::Goto(self.program.first_d.expect("a D equation")))
        } else {
            self.stage = 0;
            self.stats.integration_steps += 1;
            if self.opts.trace {
                self.trace.push(format!("     t = {}", format(self.letters[idx('t')])));
            }
            Ok(Flow::Next)
        }
    }

    fn eval(&mut self, orders: &[Order]) -> R<Num> {
        let z = self.opts.zero;
        let mut stack: Vec<Num> = Vec::with_capacity(8);
        for o in orders {
            self.stats.orders += 1;
            match o {
                Order::Const(n) => stack.push(*n),
                Order::Load(v) => {
                    let x = self.load(v)?;
                    stack.push(x);
                }
                Order::Neg => {
                    let x = stack.pop().unwrap();
                    stack.push(-x);
                }
                Order::Pow(n) => {
                    let x = stack.pop().unwrap();
                    self.stats.arithmetic += n.unsigned_abs().max(1) as u64;
                    stack.push(x.powi(*n, z).map_err(alarm)?);
                }
                Order::Func(k) => {
                    let x = stack.pop().unwrap();
                    self.stats.function_calls += 1;
                    stack.push(function(*k, x, z).map_err(alarm)?);
                }
                Order::Add | Order::Sub | Order::Mul | Order::Div => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    self.stats.arithmetic += 1;
                    let r = match o {
                        Order::Add => a.add(b, z),
                        Order::Sub => a.sub(b, z),
                        Order::Mul => a.mul(b, z),
                        _ => a.div(b, z),
                    };
                    stack.push(r.map_err(alarm)?);
                }
            }
        }
        Ok(stack.pop().unwrap_or(Num::ZERO))
    }

    /// The value of an index letter as a superscript or an equation number.
    ///
    /// "If, in the expression x|n, n has a non-integral value greater than
    /// 0.5, the computer rounds n off to the nearest integer. Negative values
    /// of n are not permitted in this connection, nor are values between zero
    /// and one half inclusive, with the exception that the value zero is
    /// permissible if assigned by the specific equation n = 0," (p. 13)
    ///
    /// Here the exception is the sign of zero: `n = 0,` gives +0, while a zero
    /// that comes out of arithmetic (n = n - 1 when n is 1) is -0, which is
    /// negative.
    fn index_value(&self, c: char, what: &str) -> R<u32> {
        let v = self.letters[idx(c)];
        if v.is_zero() {
            if v.is_negative() {
                return Err(format!(
                    "{c} is -0, a zero computed by arithmetic: as {what} zero is permissible only if assigned by the specific equation {c} = 0,"
                ));
            }
            return Ok(0);
        }
        if v.is_negative() {
            return Err(format!("{c} is negative ({}): negative values are not permitted as {what}", format(v)));
        }
        if v.value() <= 0.5 {
            return Err(format!("{c} is {}: values between zero and one half inclusive are not permitted as {what}", format(v)));
        }
        let n = (v.value() + 0.5).floor();
        if n > 1023.0 {
            return Err(format!("{c} is {}: beyond 1023", format(v)));
        }
        Ok(n as u32)
    }

    /// Resolve a variable to (letter, superscript), or None for a plain letter.
    fn resolve(&self, v: &Var) -> R<Option<(char, u32)>> {
        match v {
            Var::Letter(_) => Ok(None),
            Var::Super(c, n) => Ok(Some((*c, *n))),
            Var::Indexed(c, i) => Ok(Some((*c, self.index_value(*i, "a superscript")?))),
        }
    }

    fn name(&self, v: &Var) -> R<String> {
        Ok(match self.resolve(v)? {
            None => v.to_string(),
            Some((c, n)) => format!("{c}|{n}"),
        })
    }

    fn on_drum(&self, c: char, n: u32) -> bool {
        n >= 1 && self.drum.get(&c).is_some_and(|g| (n as usize) <= g.len())
    }

    fn load(&mut self, v: &Var) -> R<Num> {
        match self.resolve(v)? {
            None => Ok(self.letters[idx(v.letter())]),
            Some((c, n)) => {
                if self.on_drum(c, n) {
                    self.stats.drum_variable_accesses += 1;
                    return Ok(self.drum[&c][n as usize - 1]);
                }
                // A superscript variable never given a value reads as zero.
                Ok(self.core.get(&(c, n)).copied().unwrap_or(Num::ZERO))
            }
        }
    }

    fn store(&mut self, v: &Var, x: Num) -> R<()> {
        match self.resolve(v)? {
            None => {
                self.letters[idx(v.letter())] = x;
                Ok(())
            }
            Some((c, n)) => {
                if self.on_drum(c, n) {
                    self.stats.drum_variable_accesses += 1;
                    self.drum.get_mut(&c).unwrap()[n as usize - 1] = x;
                    return Ok(());
                }
                if !self.core.contains_key(&(c, n)) && self.core.len() >= self.program.core_super_limit() {
                    return Err(format!(
                        "storage exceeded: {c}|{n} would be superscript variable number {} in MS, and there is room for {}",
                        self.core.len() + 1,
                        self.program.core_super_limit()
                    ));
                }
                self.core.insert((c, n), x);
                Ok(())
            }
        }
    }

    /// "When a PRINT instruction is performed the first and last characters to
    /// appear as output are carriage returns. If more than 6 variables are
    /// called for on a single PRINT instruction, 6 will be placed on each
    /// line ... and there will be no line space between lines." (p. 6-7)
    fn print(&mut self, items: &[VarRef]) -> R<()> {
        let mut cells = Vec::new();
        for item in items {
            let name = self.name(&item.var)?;
            let v = self.load(&item.var)?;
            cells.push(format!("{name} = {}", format(v)));
            self.stats.printed_values += 1;
        }
        self.carriage_return();
        for (i, line) in cells.chunks(6).enumerate() {
            if i > 0 {
                self.carriage_return();
            }
            let mut text = String::new();
            for (j, cell) in line.iter().enumerate() {
                if j > 0 {
                    // the next tab stop: columns every 24 characters
                    let pad = 24 - (text.chars().count() % 24);
                    text.push_str(&" ".repeat(pad.max(2)));
                }
                text.push_str(cell);
            }
            self.type_text(&text);
        }
        self.carriage_return();
        Ok(())
    }

    fn type_text(&mut self, s: &str) {
        self.page.push_str(s);
        self.column += s.chars().count();
    }

    fn carriage_return(&mut self) {
        self.page.push('\n');
        self.column = 0;
    }

    /// The post-mortem: the equation in which the alarm occurred and the one
    /// before it. The wording here is made up; E-364 does not show it.
    fn type_alarm(&mut self, r: &AlarmReport) {
        let n = |x: Option<u32>| x.map(|n| n.to_string()).unwrap_or_else(|| "-".into());
        self.carriage_return();
        self.type_text(&format!("ALARM  EQUATION {}  PRECEDED BY {}", n(r.equation), n(r.preceding)));
        self.carriage_return();
    }
}

fn idx(c: char) -> usize {
    (c as u8 - b'a') as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translate::{translate, Options};

    fn page(src: &str) -> String {
        let (p, problems) = translate(src, &Options::default());
        assert!(problems.is_empty(), "{problems:?}");
        run(&p, &RunOptions::default()).page
    }

    #[test]
    fn the_first_example() {
        let out = page("a = 5,\ny = -6.3a,\nb = 0.0053 (a-y) /2ay,\nPRINT a, y, b.\nSTOP");
        // 2ay = -315 and .0053 · 36.5 = .19345: b is -.00061413, less what 24 bits lose
        assert_eq!(out, "\na = +.50000000+01       y = -.31500001+02       b = -.61412696-03\n");
    }

    #[test]
    fn prints_are_separated_by_a_line_space() {
        let out = page("a = 1,\nPRINT a. PRINT a.\nSTOP");
        assert_eq!(out, "\na = +.10000000+01\n\na = +.10000000+01\n");
    }

    #[test]
    fn a_counted_loop() {
        let out = page("n = 0,\n1 n = n + 1,\ne = n - 9.5,\nCP 1,\nPRINT n.\nSTOP");
        assert_eq!(out, "\nn = +.10000000+02\n");
    }

    #[test]
    fn cp_on_minus_zero() {
        // n - 5 with n = 5 is -0, and CP jumps on it
        let out = page("n = 5,\ne = n - 5,\nCP 1,\nPRINT e.\n1 PRINT n.\nSTOP");
        assert_eq!(out, "\nn = +.50000000+01\n");
    }

    #[test]
    fn sr_does_one_equation_and_comes_back() {
        let out = page("x = 1,\nSR 5,\nPRINT x.\nSP 6,\n5 x = x + 10,\n6 STOP");
        assert_eq!(out, "\nx = +.11000000+02\n");
    }

    #[test]
    fn an_alarm_and_equation_100() {
        let out = page("a = 0,\nb = 1/a,\n100 PRINT a.\nSTOP");
        assert_eq!(out, "\nALARM  EQUATION 102  PRECEDED BY 101\n\na = +.00000000+00\n");
    }

    #[test]
    fn a_computed_zero_index() {
        let (p, _) = translate("i = 1,\ni = i - 1,\nx = a|i,\nSTOP", &Options::default());
        let o = run(&p, &RunOptions::default());
        assert!(o.alarm.unwrap().reason.contains("-0"));
        let o = run(&p, &RunOptions { zero: ZeroModel::Positive, ..RunOptions::default() });
        assert!(o.alarm.is_none());
    }

    #[test]
    fn gill_integrates_a_circle() {
        // Dx = y, Dy = -x from (1, 0) is (cos t, -sin t): at t = 1, x = .54030231
        let src = "t = 0,\nh = 0.0625,\nx = 1,\ny = 0,\n1 Dx = y,\nDy = -x,\ne = t - 0.99,\nCP 1,\nPRINT t, x, y.\nSTOP";
        let (p, problems) = translate(src, &Options::default());
        assert!(problems.is_empty(), "{problems:?}");
        let o = run(&p, &RunOptions::default());
        let x: Vec<&str> = o.page.split_whitespace().collect();
        assert_eq!(x[2], "+.10000000+01", "{}", o.page);
        // cos 1 = .54030231 and sin 1 = .84147098, to within what 24 bits keep
        assert!(x[5].starts_with("+.540302"), "{}", o.page);
        assert!(x[8].starts_with("-.841471"), "{}", o.page);
    }
}
