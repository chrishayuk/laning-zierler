//! Every example in examples/ has a .out file with the page it types. Run each
//! and compare. A line `# options: --zero plus` at the top of an example gives
//! options, as on the command line.

use std::fs;
use std::path::Path;

use laning_zierler::machine::{run, RunOptions};
use laning_zierler::num::ZeroModel;
use laning_zierler::translate::{translate, Options};

pub fn page_of(src: &str) -> String {
    let mut opts = Options::default();
    let mut run_opts = RunOptions::default();
    for line in src.lines() {
        if let Some(rest) = line.trim().strip_prefix("# options:") {
            let words: Vec<&str> = rest.split_whitespace().collect();
            let mut i = 0;
            while i < words.len() {
                match words[i] {
                    "--zero" => {
                        let z = if words[i + 1] == "plus" { ZeroModel::Positive } else { ZeroModel::OnesComplement };
                        opts.zero = z;
                        run_opts.zero = z;
                        i += 1;
                    }
                    "--temporary-rules" => opts.temporary_rules = true,
                    other => panic!("unknown option {other}"),
                }
                i += 1;
            }
        }
    }
    let (program, problems) = translate(src, &opts);
    if !problems.is_empty() {
        let mut s = String::new();
        for p in problems {
            let (l, c) = laning_zierler::check::line_col(src, p.span.start);
            s.push_str(&format!("{}:{}: error: {}\n", l + 1, c + 1, p.message));
        }
        return s;
    }
    run(&program, &run_opts).page
}

#[test]
fn every_example_types_its_page() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut names: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "lzr"))
        .collect();
    names.sort();
    assert!(names.len() >= 10);
    let mut failures = Vec::new();
    for path in &names {
        let src = fs::read_to_string(path).unwrap();
        let got = page_of(&src);
        let out = path.with_extension("out");
        if std::env::var("LZ_BLESS").is_ok() {
            fs::write(&out, &got).unwrap();
            continue;
        }
        let want = fs::read_to_string(&out).unwrap_or_else(|_| panic!("{} has no .out file", path.display()));
        if got != want {
            failures.push(format!("{}\n--- want\n{want}\n--- got\n{got}", path.display()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
