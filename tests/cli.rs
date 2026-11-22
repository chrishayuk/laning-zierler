use std::process::Command;

fn lz(args: &[&str]) -> (String, String, bool) {
    let out = Command::new(env!("CARGO_BIN_EXE_lz")).args(args).current_dir(env!("CARGO_MANIFEST_DIR")).output().unwrap();
    (String::from_utf8_lossy(&out.stdout).into(), String::from_utf8_lossy(&out.stderr).into(), out.status.success())
}

#[test]
fn runs_a_tape() {
    let (out, _, ok) = lz(&["examples/1954-first-equations.lzr"]);
    assert!(ok);
    assert!(out.contains("a = +.50000000+01"));
}

#[test]
fn knuths_tpk_against_the_1954_rules_and_the_zero_models() {
    let (out, err, ok) = lz(&["examples/1976-knuth-tpk.lzr"]);
    assert!(!ok);
    assert!(out.contains("ALARM  EQUATION 2"));
    assert!(err.contains("-0"));
    let (out, _, ok) = lz(&["--zero", "plus", "examples/1976-knuth-tpk.lzr"]);
    assert!(ok);
    assert!(out.contains("i = +.00000000+00       y = +.60000000+01"));
}

#[test]
fn the_temporary_rules_of_january_1954() {
    let (_, err, ok) = lz(&["--temporary-rules", "examples/1976-knuth-tpk.lzr"]);
    assert!(!ok);
    assert!(err.contains("not yet in operation"));
}

#[test]
fn translate_lists_the_blocks() {
    let (out, _, ok) = lz(&["--translate", "examples/1954-first-equations.lzr"]);
    assert!(ok);
    assert!(out.contains("103  b = 0.0053 (a-y) /2ay,"));
    assert!(out.contains("       div"));
}

#[test]
fn check_finds_the_innocuous_change() {
    let (out, _, _) = lz(&["--check", "examples/1954-innocuous-change.lzr"]);
    assert!(out.contains("auxiliary computation"), "{out}");
}

#[test]
fn tapes_are_read_in_turn() {
    let dir = std::env::temp_dir();
    let data = dir.join("lz-data.lzr");
    let prog = dir.join("lz-prog.lzr");
    std::fs::write(&data, "a = 21,\n").unwrap();
    std::fs::write(&prog, "b = 2a,\nPRINT b.\nSTOP\n").unwrap();
    let (out, _, ok) = lz(&[data.to_str().unwrap(), prog.to_str().unwrap()]);
    assert!(ok);
    assert!(out.contains("b = +.42000000+02"));
}
