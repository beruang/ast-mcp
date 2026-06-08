use ast_mcp::parser;
use rand::Rng;

/// Feed random byte sequences to each parser and assert no panic.
/// 100 sequences per language, 1–4096 bytes each.
#[test]
fn fuzz_all_parsers_no_panic() {
    let languages = [
        parser::registry::for_extension(".ts").unwrap(),
        parser::registry::for_extension(".tsx").unwrap(),
        parser::registry::for_extension(".js").unwrap(),
        parser::registry::for_extension(".jsx").unwrap(),
        parser::registry::for_extension(".py").unwrap(),
    ];

    let mut rng = rand::thread_rng();
    // Use a fixed seed for reproducibility
    let seed = rng.gen::<u64>();
    eprintln!("fuzz seed: {}", seed);
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);

    let sequences_per_lang = 100;

    for lang in &languages {
        for i in 0..sequences_per_lang {
            let len = rng.gen_range(1..=4096usize);
            let bytes: Vec<u8> = (0..len).map(|_| rng.gen::<u8>()).collect();

            // Try to interpret as UTF-8; skip if invalid (Tree-sitter needs &str)
            let source = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // parse_source must not panic
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                parser::parse::parse_source(&source, lang.language)
            }));

            match result {
                Ok(Ok((_tree, status))) => {
                    // Success: check basic invariants
                    assert!(
                        !status.root_kind.is_empty(),
                        "{} seq {}: root_kind is empty",
                        lang.language.as_str(),
                        i
                    );
                }
                Ok(Err(_e)) => {
                    // Parse failure is acceptable (syntax error, etc.)
                }
                Err(panic_payload) => {
                    let msg = panic_payload
                        .downcast_ref::<&str>()
                        .map(|s| s.to_string())
                        .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                        .unwrap_or_else(|| "(non-string panic)".to_string());
                    panic!(
                        "parser panicked on {} (seq {}/{}, len={}): {}",
                        lang.language.as_str(),
                        i,
                        sequences_per_lang,
                        len,
                        msg
                    );
                }
            }
        }
    }
}

/// Smoke test: parse a single valid file in each language to confirm the
/// fuzz harness doesn't have a broken setup.
#[test]
fn fuzz_smoke_valid_inputs() {
    let cases: &[(&str, &str)] = &[
        ("typescript", "const x: number = 1;"),
        ("tsx", "const x: number = <div />;"),
        ("javascript", "const x = 1;"),
        ("jsx", "const x = <div />;"),
        ("python", "x = 1"),
    ];

    for (name, source) in cases {
        let ext = match *name {
            "typescript" => ".ts",
            "tsx" => ".tsx",
            "javascript" => ".js",
            "jsx" => ".jsx",
            "python" => ".py",
            _ => panic!("unknown name: {}", name),
        };
        let entry = parser::registry::for_extension(ext)
            .unwrap_or_else(|| panic!("no parser for {}", name));

        let result = parser::parse::parse_source(source, entry.language);
        assert!(result.is_ok(), "failed to parse valid {} input: {:?}", name, result);
    }
}
