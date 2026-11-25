//! The one page of output from the 1954 system that we have found: the first
//! page typed by the lead-pursuit program, from Laning's letter to Knuth of
//! 2 July 1976 (examples/1954-lead-pursuit.whirlwind). Run the program here
//! and compare every value on that page, in order.
//!
//! The page ends at t = .6, before the program first changes its step h (after
//! t = .67), so it checks the run only that far. The rest of the run is
//! checked against this reconstruction's own baseline, the .out file, by
//! tests/examples.rs; here the run must only reach its STOP without an alarm.
//!
//! The two disagree in the last digits, and the test allows for it. Every
//! value agrees to within 4.2 parts in ten million except the two of a|0, from
//! the arcsine (F^5), which are out by up to 7.2 parts in a million. Whirlwind
//! typed a zero set by `t = 0,` as +.16155871-27, which here is 0, and a zero
//! exponent as -00, which here is +00.

use std::fs;
use std::path::Path;

use laning_zierler::machine::{run, Outcome, RunOptions};
use laning_zierler::translate::{translate, Options};

/// A number as typed, `+.68899998-02`: its signed mantissa and the three
/// characters of its exponent.
fn parse(typed: &str) -> (f64, &str) {
    let sign = if typed.starts_with('-') { -1.0 } else { 1.0 };
    let body = &typed[2..]; // after the sign and the point
    let mantissa: f64 = format!("0.{}", &body[..8]).parse().unwrap();
    (sign * mantissa, &body[8..])
}

/// Whether two exponents agree. A `?` in the 1954 one is a digit lost off the
/// edge of the page, and every other character must match; -00 and +00 are
/// the same exponent.
fn same_exponent(then: &str, now: &str) -> bool {
    if then.contains('?') {
        return then.len() == now.len()
            && then.chars().zip(now.chars()).all(|(a, b)| a == '?' || a == b);
    }
    then.parse::<i32>().unwrap() == now.parse::<i32>().unwrap()
}

fn whirlwind_page() -> Vec<(String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/1954-lead-pursuit.whirlwind");
    let text = fs::read_to_string(path).unwrap();
    let mut values = Vec::new();
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        for item in line.split_whitespace() {
            let (name, value) = item.split_once('=').unwrap();
            values.push((name.to_string(), value.to_string()));
        }
    }
    values
}

fn reconstruction_run() -> Outcome {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/1954-lead-pursuit.lzr");
    let src = fs::read_to_string(path).unwrap();
    let (program, problems) = translate(&src, &Options::default());
    assert!(problems.is_empty(), "the tape does not translate");
    run(&program, &RunOptions::default())
}

fn values_of(page: &str) -> Vec<(String, String)> {
    let words: Vec<&str> = page.split_whitespace().collect();
    let mut values = Vec::new();
    for i in 1..words.len().saturating_sub(1) {
        if words[i] == "=" {
            values.push((words[i - 1].to_string(), words[i + 1].to_string()));
        }
    }
    values
}

#[test]
fn the_lead_pursuit_program_types_what_whirlwind_typed() {
    let outcome = reconstruction_run();
    assert!(outcome.alarm.is_none(), "the run ends in an alarm");
    assert!(!outcome.limit_reached, "the run does not reach its STOP");
    let then = whirlwind_page();
    let now = values_of(&outcome.page);
    assert_eq!(then.len(), 62);
    assert!(now.len() >= then.len());
    let mut worst = 0.0f64;
    for (i, ((name_then, typed_then), (name_now, typed_now))) in then.iter().zip(&now).enumerate() {
        let what = format!("value {i}: {name_then} is {typed_then} against {typed_now}");
        assert_eq!(name_then, name_now, "value {i}: the pages print different variables");
        let (m_then, e_then) = parse(typed_then);
        let (m_now, e_now) = parse(typed_now);
        if m_now == 0.0 {
            // A zero: Whirlwind typed +.16155871-27.
            assert_eq!(typed_then, "+.16155871-27", "{what}");
            continue;
        }
        assert!(same_exponent(e_then, e_now), "{what}");
        let rel = ((m_then - m_now) / m_now).abs();
        worst = worst.max(rel);
        // Every value agrees to within one part in two million (the 24-bit
        // mantissa is good to one in 17 million) except a|0, which comes from
        // the arcsine: the 1954 function routines are not known.
        let allowed = if name_then == "a|0" { 1e-5 } else { 5e-7 };
        assert!(rel < allowed, "{what}");
    }
    eprintln!("worst relative difference {worst:e}");
}
