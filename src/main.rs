//! `lz`: feed tapes to the Laning and Zierler system and get the typed page.

use std::io::{IsTerminal, Read, Write};
use std::process::ExitCode;

use laning_zierler::check::{check, render, Severity};
use laning_zierler::machine::{run, RunOptions};
use laning_zierler::num::ZeroModel;
use laning_zierler::translate::{listing, translate, Options};

const USAGE: &str = "\
lz: the algebraic system of Laning and Zierler (Whirlwind I, 1954)

usage: lz [options] [tape ...]

Each tape is read in turn, as a data tape and then the program tape were fed
to the reader after a stop code. With no tape, standard input is read.

  --translate          print each equation's number and its block of orders; do not run
  --check              read the tape and report problems and warnings; do not run
  --trace              print each equation as it is carried out (to standard error)
  --stats              count what the run cost (to standard error)
  --zero ones|plus     how a zero gets its sign: one's complement (default; n - n is -0)
                       or always +0
  --temporary-rules    the restrictions of January 1954 (E-364, section 18):
                       no x|N, and no SR or CR to an equation with a function
  --limit N            stop after N equations (default 5000000)
  -h, --help           this
";

fn main() -> ExitCode {
    let mut files = Vec::new();
    let mut mode = "run";
    let mut opts = Options::default();
    let mut run_opts = RunOptions::default();
    let mut stats = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "--translate" => mode = "translate",
            "--check" => mode = "check",
            "--trace" => run_opts.trace = true,
            "--stats" => stats = true,
            "--temporary-rules" => opts.temporary_rules = true,
            "--zero" => {
                let z = match args.next().as_deref() {
                    Some("ones") | Some("ones-complement") => ZeroModel::OnesComplement,
                    Some("plus") | Some("positive") => ZeroModel::Positive,
                    other => {
                        eprintln!("lz: --zero is ones or plus, not {}", other.unwrap_or("nothing"));
                        return ExitCode::from(2);
                    }
                };
                opts.zero = z;
                run_opts.zero = z;
            }
            "--limit" => match args.next().and_then(|n| n.parse().ok()) {
                Some(n) => run_opts.limit = n,
                None => {
                    eprintln!("lz: --limit needs a number");
                    return ExitCode::from(2);
                }
            },
            s if s.starts_with('-') && s != "-" => {
                eprintln!("lz: unknown option {s}\n\n{USAGE}");
                return ExitCode::from(2);
            }
            _ => files.push(a),
        }
    }

    let mut src = String::new();
    let mut name = String::from("<stdin>");
    if files.is_empty() || files == ["-"] {
        if std::io::stdin().is_terminal() {
            eprintln!("lz: type the tape, then end it with ctrl-D (lz --help for more)");
        }
        if std::io::stdin().read_to_string(&mut src).is_err() {
            eprintln!("lz: cannot read standard input");
            return ExitCode::from(2);
        }
    } else {
        for (i, f) in files.iter().enumerate() {
            match std::fs::read_to_string(f) {
                Ok(text) => {
                    if i > 0 && !src.ends_with('\n') {
                        src.push('\n');
                    }
                    src.push_str(&text);
                }
                Err(e) => {
                    eprintln!("lz: {f}: {e}");
                    return ExitCode::from(2);
                }
            }
        }
        name = files.join("+");
    }

    if mode == "check" {
        let diags = check(&src, &opts);
        print!("{}", render(&src, &name, &diags));
        let errors = diags.iter().filter(|d| d.severity == Severity::Error).count();
        return if errors > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS };
    }

    let (program, problems) = translate(&src, &opts);
    if !problems.is_empty() {
        let diags: Vec<_> = problems
            .into_iter()
            .map(|p| laning_zierler::check::Diagnostic { severity: Severity::Error, message: p.message, span: p.span })
            .collect();
        eprint!("{}", render(&src, &name, &diags));
        return ExitCode::FAILURE;
    }
    if mode == "translate" {
        print!("{}", listing(&src, &program));
        return ExitCode::SUCCESS;
    }

    let outcome = run(&program, &run_opts);
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(outcome.page.as_bytes());
    let _ = stdout.flush();
    if run_opts.trace {
        for line in &outcome.trace {
            eprintln!("{line}");
        }
    }
    if let Some(a) = &outcome.alarm {
        eprintln!("lz: alarm: {}", a.reason);
    }
    if outcome.limit_reached {
        eprintln!("lz: stopped after {} equations (--limit)", run_opts.limit);
    }
    if stats {
        let s = &outcome.stats;
        eprintln!("equations carried out      {}", s.statements);
        eprintln!("blocks brought from drum   {}", s.block_transfers);
        eprintln!("interpretive orders        {}", s.orders);
        eprintln!("floating-point operations  {}", s.arithmetic);
        eprintln!("function subroutine calls  {}", s.function_calls);
        eprintln!("drum variable accesses     {}  (about 30 ms each, E-364 p. 16: {:.1} s)", s.drum_variable_accesses, s.drum_variable_accesses as f64 * 0.030);
        eprintln!("integration steps          {}", s.integration_steps);
        eprintln!("values printed             {}", s.printed_values);
    }
    if outcome.alarm.is_some() || outcome.limit_reached {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
