//! Keeps the tutorial honest. In tutorial/*.md, a ```lzr block followed at
//! once by a ```page block is a sitting: a tape fed to `lz`, which must type
//! exactly what the page block says. The info string after `lzr` gives
//! command-line options (```lzr --zero plus). The page is what `lz` prints on
//! standard output, followed by anything it prints on standard error (an
//! alarm explained, a translation problem). A ```lzr block followed by
//! anything else is a fragment and is not run.
//!
//! Run with LZ_BLESS=1 to rewrite the page blocks from real runs.

#![cfg(not(target_os = "wasi"))]

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

struct Block {
    info: String,
    text: String,
    start: usize,
    end: usize,
    adjoins: bool,
}

fn blocks(src: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut pos = 0;
    let mut open: Option<(String, usize)> = None;
    let mut prose_since_last = true;
    for line in src.split_inclusive('\n') {
        let fence = line.trim_end().strip_prefix("```");
        match (&open, fence) {
            (None, Some(info)) => open = Some((info.to_string(), pos + line.len())),
            (Some((info, start)), Some("")) => {
                out.push(Block {
                    info: info.clone(),
                    text: src[*start..pos].to_string(),
                    start: *start,
                    end: pos,
                    adjoins: !prose_since_last,
                });
                open = None;
                prose_since_last = false;
            }
            (None, None) if !line.trim().is_empty() => prose_since_last = true,
            _ => {}
        }
        pos += line.len();
    }
    assert!(open.is_none(), "a code block is not closed");
    out
}

/// What `lz` types for a tape on standard input, then what it says on
/// standard error.
fn page(options: &str, tape: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lz"))
        .args(options.split_whitespace())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(tape.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    let mut s = String::from_utf8_lossy(&output.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&output.stderr));
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

#[test]
fn every_sitting_in_the_tutorial_types_what_the_tutorial_says() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tutorial");
    let bless = std::env::var("LZ_BLESS").is_ok();
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    files.sort();
    assert!(files.len() >= 12, "the tutorial has chapters");
    let mut sittings = 0;
    let mut failures = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).unwrap();
        let bs = blocks(&src);
        let mut edits: Vec<(usize, usize, String)> = Vec::new();
        for pair in bs.windows(2) {
            let (tape, want) = (&pair[0], &pair[1]);
            let Some(options) = tape.info.strip_prefix("lzr") else { continue };
            if want.info != "page" || !want.adjoins {
                continue;
            }
            sittings += 1;
            let got = page(options, &tape.text);
            if got != want.text {
                if bless {
                    edits.push((want.start, want.end, got));
                } else {
                    failures.push(format!(
                        "{}:\n{}\n--- the tutorial says\n{}--- lz types\n{}",
                        path.file_name().unwrap().to_string_lossy(),
                        tape.text,
                        want.text,
                        got
                    ));
                }
            }
        }
        if bless && !edits.is_empty() {
            let mut out = src.clone();
            for (start, end, text) in edits.into_iter().rev() {
                out.replace_range(start..end, &text);
            }
            fs::write(path, out).unwrap();
        }
    }
    assert!(sittings >= 60, "only {sittings} sittings");
    assert!(failures.is_empty(), "{} of {sittings} sittings differ:\n\n{}", failures.len(), failures.join("\n\n"));
}
