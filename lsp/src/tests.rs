use super::*;

const URI: &str = "file:///test.lzr";
/// E-364 p. 18: the two differential equations.
const PROGRAM: &str = "\
# E-364 p. 18
    t = 2,
    h = 0.5,
    y|1 = 0,
    y|2 = 0,
1   PRINT t, y|1, y|2.
    e = 9.7 - t,
    CP 3,
2   D y|1 = y|2 + 1,
    D y|2 = - y|1,
    e = -e,
    CP 2,
    SP 1,
3   STOP
";

fn server_with(text: &str) -> (Server, Vec<Value>) {
    let mut server = Server::new();
    let out = server.handle(&json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": URI, "languageId": "laning-zierler", "version": 1, "text": text } } }));
    (server, out)
}

fn ask(server: &mut Server, method: &str, line: u32, character: u32, extra: Value) -> Value {
    let mut params = json!({ "textDocument": { "uri": URI }, "position": { "line": line, "character": character } });
    for (key, value) in extra.as_object().into_iter().flatten() {
        params[key] = value.clone();
    }
    let mut out = server.handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }));
    assert_eq!(out.len(), 1);
    out.remove(0)
}

fn hover_text(server: &mut Server, line: u32, character: u32) -> String {
    ask(server, "textDocument/hover", line, character, json!({}))["result"]["contents"]["value"].as_str().unwrap_or("").to_string()
}

#[test]
fn opening_publishes_no_errors_for_the_manuals_program() {
    let (_, out) = server_with(PROGRAM);
    assert_eq!(out[0]["method"], "textDocument/publishDiagnostics");
    let errors: Vec<&Value> = out[0]["params"]["diagnostics"].as_array().unwrap().iter().filter(|d| d["severity"] == 1).collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn problems_are_published_with_their_place() {
    let (_, out) = server_with("x = 5\nSTOP");
    let d = &out[0]["params"]["diagnostics"][0];
    assert_eq!(d["severity"], 1);
    assert_eq!(d["source"], "lz");
    assert!(d["message"].as_str().unwrap().contains("comma"), "{d}");
}

#[test]
fn hover_explains_words_numbers_and_letters() {
    let (mut s, _) = server_with(PROGRAM);
    assert!(hover_text(&mut s, 7, 4).contains("negative"));
    let eq3 = hover_text(&mut s, 7, 7);
    assert!(eq3.contains("Equation 3") && eq3.contains("STOP"), "{eq3}");
    assert!(hover_text(&mut s, 8, 4).contains("Gill"));
    let y1 = hover_text(&mut s, 8, 6);
    assert!(y1.contains("superscript variable") && y1.contains("y|1"), "{y1}");
    assert!(hover_text(&mut s, 1, 4).contains("independent variable"));
    let eq = hover_text(&mut s, 6, 6);
    assert!(eq.contains("Equation 105") && eq.contains("store e") && eq.contains("sub"), "{eq}");
    let number = hover_text(&mut s, 6, 9);
    assert!(number.contains("24, 6") && number.contains("+.96999998+01"), "{number}");
    assert!(hover_text(&mut s, 0, 3).contains("note"));
}

#[test]
fn hover_on_functions_and_the_slash() {
    let (mut s, _) = server_with("x = F^2(y)/a/b,\nSTOP");
    assert!(hover_text(&mut s, 0, 4).contains("sine"));
    assert!(hover_text(&mut s, 0, 6).contains("sine"));
    assert!(hover_text(&mut s, 0, 10).contains("ac divided by b"));
}

#[test]
fn definition_references_and_highlights_of_equation_numbers() {
    let (mut s, _) = server_with(PROGRAM);
    let def = ask(&mut s, "textDocument/definition", 7, 7, json!({}));
    assert_eq!(def["result"]["range"]["start"]["line"], 13);
    let refs = ask(&mut s, "textDocument/references", 12, 7, json!({ "context": { "includeDeclaration": true } }));
    assert_eq!(refs["result"].as_array().unwrap().len(), 2);
    let e = ask(&mut s, "textDocument/definition", 10, 9, json!({}));
    assert_eq!(e["result"]["range"]["start"]["line"], 6);
    let hl = ask(&mut s, "textDocument/documentHighlight", 6, 4, json!({}));
    assert_eq!(hl["result"].as_array().unwrap().len(), 3);
}

#[test]
fn rename_an_equation_number_or_a_letter() {
    let (mut s, _) = server_with(PROGRAM);
    let r = ask(&mut s, "textDocument/rename", 13, 0, json!({ "newName": "9" }));
    assert_eq!(r["result"]["changes"][URI].as_array().unwrap().len(), 2);
    let clash = ask(&mut s, "textDocument/rename", 13, 0, json!({ "newName": "1" }));
    assert!(clash["error"]["message"].as_str().unwrap().contains("already"));
    let big = ask(&mut s, "textDocument/rename", 13, 0, json!({ "newName": "150" }));
    assert!(big["error"].is_object());
    let e = ask(&mut s, "textDocument/rename", 6, 4, json!({ "newName": "k" }));
    assert_eq!(e["result"]["changes"][URI].as_array().unwrap().len(), 3);
    let t = ask(&mut s, "textDocument/rename", 1, 4, json!({ "newName": "u" }));
    assert!(t["error"]["message"].as_str().unwrap().contains("independent"));
    let prepared = ask(&mut s, "textDocument/prepareRename", 4, 0, json!({}));
    assert!(prepared["result"].is_null());
}

#[test]
fn inlay_hints_give_the_translators_numbers() {
    let (mut s, _) = server_with(PROGRAM);
    let hints = ask(&mut s, "textDocument/inlayHint", 0, 0, json!({ "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 14, "character": 0 } } }));
    let hints = hints["result"].as_array().unwrap();
    let labels: Vec<&str> = hints.iter().map(|h| h["label"].as_str().unwrap()).collect();
    assert_eq!(labels, vec!["101", "102", "103", "104", "105", "106", "107"]);
    assert_eq!(hints[0]["position"]["line"], 1);
}

#[test]
fn completion_in_its_places() {
    let (mut s, _) = server_with("1 x = 1,\nSP \nSTOP");
    let labels = |v: &Value| v["result"]["items"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    let after_sp = labels(&ask(&mut s, "textDocument/completion", 1, 3, json!({})));
    assert!(after_sp.contains(&"1".to_string()), "{after_sp:?}");
    let start = labels(&ask(&mut s, "textDocument/completion", 0, 0, json!({})));
    assert!(start.contains(&"PRINT".to_string()) && start.contains(&"D".to_string()), "{start:?}");
    let (mut s, _) = server_with("x = F\nSTOP");
    let f = labels(&ask(&mut s, "textDocument/completion", 0, 5, json!({})));
    assert!(f.contains(&"F^23".to_string()), "{f:?}");
    let (mut s, _) = server_with("x = a + \nSTOP");
    let expr = labels(&ask(&mut s, "textDocument/completion", 0, 8, json!({})));
    assert!(expr.contains(&"F^1".to_string()) && expr.contains(&"x".to_string()), "{expr:?}");
}

#[test]
fn signature_help_inside_a_function() {
    let (mut s, _) = server_with("x = F^2(y + (1)),\nSTOP");
    let help = ask(&mut s, "textDocument/signatureHelp", 0, 9, json!({}));
    assert_eq!(help["result"]["signatures"][0]["label"], "F²(x)");
    let inner = ask(&mut s, "textDocument/signatureHelp", 0, 14, json!({}));
    assert!(inner["result"].is_null());
    let outside = ask(&mut s, "textDocument/signatureHelp", 0, 2, json!({}));
    assert!(outside["result"].is_null());
}

#[test]
fn symbols_folding_and_semantic_tokens() {
    let (mut s, _) = server_with(PROGRAM);
    let symbols = ask(&mut s, "textDocument/documentSymbol", 0, 0, json!({}));
    let names: Vec<&str> = symbols["result"].as_array().unwrap().iter().map(|x| x["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["Equation 1", "2  Dy|1", "Dy|2", "3  STOP"]);
    let folds = ask(&mut s, "textDocument/foldingRange", 0, 0, json!({}));
    assert_eq!(folds["result"][0]["startLine"], 8);
    let tokens = ask(&mut s, "textDocument/semanticTokens/full", 0, 0, json!({}));
    let data = tokens["result"]["data"].as_array().unwrap();
    assert!(!data.is_empty() && data.len() % 5 == 0);
    // the first token is the note, a comment
    assert_eq!(data[3], 0);
}

#[test]
fn the_flexowriters_characters_and_the_half() {
    let (mut s, _) = server_with("x = a^2 * b^-3,\nSTOP");
    let f = ask(&mut s, "textDocument/formatting", 0, 0, json!({}));
    let texts: Vec<&str> = f["result"].as_array().unwrap().iter().map(|e| e["newText"].as_str().unwrap()).collect();
    assert_eq!(texts, vec!["²", "·", "⁻³"]);

    let (mut s, out) = server_with("n = 0,\n1 n = n + 1,\ne = n - 10,\nCP 1,\nSTOP");
    assert!(out[0]["params"]["diagnostics"].as_array().unwrap().iter().any(|d| d["severity"] == 4));
    let actions = ask(&mut s, "textDocument/codeAction", 0, 0,
        json!({ "range": { "start": { "line": 3, "character": 0 }, "end": { "line": 3, "character": 4 } } }));
    let fix = &actions["result"][0];
    assert!(fix["title"].as_str().unwrap().contains("9.5"), "{fix}");
    assert_eq!(fix["edit"]["changes"][URI][0]["range"]["start"], json!({ "line": 2, "character": 8 }));
}

#[test]
fn utf16_columns_after_superscripts() {
    // ² is one UTF-16 unit, but more than one byte
    let (mut s, _) = server_with("a = 5²·b,\nSTOP");
    assert!(hover_text(&mut s, 0, 7).contains("letter"));
    assert!(hover_text(&mut s, 0, 6).contains("times"));
}

#[test]
fn lifecycle_and_unknown_methods() {
    let mut s = Server::new();
    let init = s.handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }));
    assert_eq!(init[0]["result"]["serverInfo"]["name"], "lz-lsp");
    assert_eq!(init[0]["result"]["capabilities"]["inlayHintProvider"], true);
    let other = s.handle(&json!({ "jsonrpc": "2.0", "id": 2, "method": "workspace/symbol", "params": { "query": "" } }));
    assert_eq!(other[0]["error"]["code"], -32601);
    let unopened = s.handle(&json!({ "jsonrpc": "2.0", "id": 3, "method": "textDocument/hover",
        "params": { "textDocument": { "uri": "file:///none.lzr" }, "position": { "line": 0, "character": 0 } } }));
    assert!(unopened[0]["result"].is_null());
    s.handle(&json!({ "jsonrpc": "2.0", "id": 4, "method": "shutdown" }));
    s.handle(&json!({ "jsonrpc": "2.0", "method": "exit" }));
    assert!(s.shutdown && s.exit);
}

/// Every request at every position of every example, and of every prefix of
/// one: nothing may panic.
#[test]
fn every_request_everywhere_in_every_example() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples");
    let mut texts: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "lzr"))
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect();
    let tpk = texts.iter().find(|t| t.contains("TPK")).cloned().unwrap_or_default();
    for cut in (0..tpk.len()).filter(|&i| tpk.is_char_boundary(i)).step_by(7) {
        texts.push(tpk[..cut].to_string());
    }
    for text in &texts {
        let (mut s, out) = server_with(text);
        assert_eq!(out[0]["method"], "textDocument/publishDiagnostics");
        for (line, l) in text.split('\n').enumerate() {
            for character in 0..=l.encode_utf16().count() as u32 {
                for method in ["textDocument/hover", "textDocument/definition", "textDocument/completion", "textDocument/signatureHelp", "textDocument/documentHighlight", "textDocument/prepareRename"] {
                    ask(&mut s, method, line as u32, character, json!({}));
                }
            }
            ask(&mut s, "textDocument/codeAction", 0, 0,
                json!({ "range": { "start": { "line": line, "character": 0 }, "end": { "line": line, "character": 99 } } }));
        }
        for method in ["textDocument/documentSymbol", "textDocument/foldingRange", "textDocument/semanticTokens/full", "textDocument/formatting", "textDocument/inlayHint"] {
            ask(&mut s, method, 0, 0, json!({}));
        }
    }
}
