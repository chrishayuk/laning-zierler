//! Keeps the tutorial honest.
//!
//! In tutorial/*.md, a ```lzr block followed at once by a ```page block is a
//! sitting: a tape fed to `lz`, which must type exactly what the page block
//! says. The info string after `lzr` gives command-line options (```lzr
//! --zero plus). Two or more ```lzr blocks one after another, then a ```page
//! block, are tapes read in turn (`lz data.lzr program.lzr`); the options are
//! the first block's. A ```lzr block followed by anything else is a fragment
//! and is not run.
//!
//! The page is everything `lz` writes, standard output and standard error
//! through one pipe, in the order it wrote it. `lz` must exit with a failure
//! exactly when the page has an error, an alarm or the limit in it.
//!
//! The test also checks that every relative link in the tutorial, and every
//! `examples/...lzr` it names, exists.
//!
//! Run with LZ_BLESS=1 to rewrite the page blocks from real runs.

#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The number of chapters. Change it when a chapter is added.
const CHAPTERS: usize = 16;

#[derive(Debug)]
struct Block {
    info: String,
    text: String,
    start: usize,
    end: usize,
    /// Nothing but blank lines between this block and the one before it.
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

/// A sitting: the tapes (with the first block's options) and the page.
struct Sitting<'a> {
    options: &'a str,
    tapes: Vec<&'a str>,
    page: &'a Block,
}

fn sittings(bs: &[Block]) -> Vec<Sitting<'_>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bs.len() {
        let Some(options) = bs[i].info.strip_prefix("lzr") else {
            i += 1;
            continue;
        };
        let mut tapes = vec![bs[i].text.as_str()];
        let mut j = i + 1;
        while j < bs.len() && bs[j].adjoins && bs[j].info == "lzr" {
            tapes.push(&bs[j].text);
            j += 1;
        }
        if j < bs.len() && bs[j].adjoins && bs[j].info == "page" {
            out.push(Sitting { options, tapes, page: &bs[j] });
            i = j + 1;
        } else {
            i = j;
        }
    }
    out
}

/// What `lz` writes for the tapes, standard output and standard error in one
/// pipe, and whether it succeeded. One tape goes on standard input; several
/// are written to files and named, so `lz` reads them in turn.
fn run(options: &str, tapes: &[&str], scratch: &Path) -> (String, bool) {
    let mut args: Vec<String> = options.split_whitespace().map(String::from).collect();
    let mut files: Vec<PathBuf> = Vec::new();
    if tapes.len() > 1 {
        for (k, tape) in tapes.iter().enumerate() {
            let f = scratch.join(format!("tape{k}.lzr"));
            fs::write(&f, tape).unwrap();
            files.push(f);
        }
        args.extend(files.iter().map(|f| f.to_string_lossy().into_owned()));
    }
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("exec \"$0\" \"$@\" 2>&1")
        .arg(env!("CARGO_BIN_EXE_lz"))
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let input = if tapes.len() == 1 { tapes[0] } else { "" };
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    let mut s = String::from_utf8_lossy(&output.stdout).into_owned();
    if !s.ends_with('\n') {
        s.push('\n');
    }
    (s, output.status.success())
}

fn chapters() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tutorial");
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_sitting_in_the_tutorial_types_what_the_tutorial_says() {
    let bless = std::env::var("LZ_BLESS").is_ok();
    let files = chapters();
    assert_eq!(files.len(), CHAPTERS, "the tutorial has {CHAPTERS} chapters");
    let scratch = std::env::temp_dir().join(format!("lz-tutorial-{}", std::process::id()));
    fs::create_dir_all(&scratch).unwrap();
    let mut count = 0;
    let mut failures = Vec::new();
    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let src = fs::read_to_string(path).unwrap();
        let bs = blocks(&src);
        let mut edits: Vec<(usize, usize, String)> = Vec::new();
        for s in sittings(&bs) {
            count += 1;
            let (got, ok) = run(s.options, &s.tapes, &scratch);
            let failed = got.contains("error:") || got.contains("ALARM") || got.contains("lz: stopped");
            if ok == failed {
                failures.push(format!("{name}:\n{}\n--- lz exited with {}, and the page says\n{got}", s.tapes.join("\n"), if ok { "success" } else { "failure" }));
            }
            if got != s.page.text {
                if bless {
                    edits.push((s.page.start, s.page.end, got));
                } else {
                    failures.push(format!("{name}:\n{}\n--- the tutorial says\n{}--- lz types\n{got}", s.tapes.join("\n"), s.page.text));
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
    let _ = fs::remove_dir_all(&scratch);
    assert!(count >= 90, "only {count} sittings");
    assert!(failures.is_empty(), "{} of {count} sittings are wrong:\n\n{}", failures.len(), failures.join("\n\n"));
}

#[test]
fn every_link_and_example_the_tutorial_names_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut missing = Vec::new();
    for path in chapters() {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let src = fs::read_to_string(&path).unwrap();
        // [text](target)
        let mut rest = src.as_str();
        while let Some(i) = rest.find("](") {
            rest = &rest[i + 2..];
            let Some(j) = rest.find(')') else { break };
            let target = &rest[..j];
            rest = &rest[j..];
            if target.starts_with("http") || target.starts_with('#') || target.is_empty() {
                continue;
            }
            let file = target.split('#').next().unwrap();
            if !root.join("tutorial").join(file).exists() {
                missing.push(format!("{name}: link to {target}"));
            }
        }
        // examples/name.lzr, anywhere
        for (k, _) in src.match_indices("examples/") {
            let tail: String = src[k..].chars().take_while(|c| c.is_alphanumeric() || "/-_.".contains(*c)).collect();
            let tail = tail.trim_end_matches('.');
            if tail.ends_with(".lzr") && !root.join(tail).exists() {
                missing.push(format!("{name}: {tail}"));
            }
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

/// The reader of blocks reads them as this file's opening comment says.
#[test]
fn the_blocks_are_read_as_the_tutorial_says_they_are() {
    let doc = "Prose.\n\n```lzr\na = 1,\n```\n```page\none\n```\n\nProse.\n\n```lzr\nfragment\n```\n\nProse, so the next page is not this tape's.\n\n```page\nstray\n```\n\n```lzr --zero plus\nfirst,\n```\n\n```lzr\nsecond,\n```\n```page\ntwo\n```\n";
    let bs = blocks(doc);
    assert_eq!(bs.len(), 7);
    let ss = sittings(&bs);
    assert_eq!(ss.len(), 2, "a fragment and a stray page are not sittings");
    assert_eq!(ss[0].options, "");
    assert_eq!(ss[0].tapes, vec!["a = 1,\n"]);
    assert_eq!(ss[0].page.text, "one\n");
    assert_eq!(ss[1].options, " --zero plus");
    assert_eq!(ss[1].tapes, vec!["first,\n", "second,\n"]);
    assert_eq!(ss[1].page.text, "two\n");
}
