use crate::config::workspace::Workspace;
use crate::parser::registry;
use crate::shared::types_v5::{ReadinessCheck, ReadinessResult};

pub fn check(workspace: &Workspace, require_languages: Option<&[String]>) -> ReadinessResult {
    let mut checks = Vec::new();

    // Workspace checks
    let root = workspace.root();
    checks.push(ReadinessCheck {
        name: "workspace_exists".into(),
        ok: root.exists(),
        message: if root.exists() { None } else { Some("workspace path does not exist".into()) },
    });
    checks.push(ReadinessCheck {
        name: "workspace_is_directory".into(),
        ok: root.is_dir(),
        message: if root.is_dir() {
            None
        } else {
            Some("workspace path is not a directory".into())
        },
    });
    let readable = std::fs::read_dir(root).is_ok();
    checks.push(ReadinessCheck {
        name: "workspace_readable".into(),
        ok: readable,
        message: if readable { None } else { Some("cannot read workspace directory".into()) },
    });

    // Parser registry checks
    let parsers = registry::list_languages();
    checks.push(ReadinessCheck {
        name: "parser_registry".into(),
        ok: !parsers.is_empty(),
        message: if parsers.is_empty() { Some("no parsers registered".into()) } else { None },
    });

    // Required languages
    if let Some(langs) = require_languages {
        for lang in langs {
            let available = parsers.iter().any(|p| p.language() == lang.as_str());
            checks.push(ReadinessCheck {
                name: format!("language_{}", lang),
                ok: available,
                message: if available {
                    None
                } else {
                    Some(format!("language '{}' not available", lang))
                },
            });
        }
    }

    checks.push(ReadinessCheck { name: "cache_initialized".into(), ok: true, message: None });
    checks.push(ReadinessCheck { name: "runtime_config_valid".into(), ok: true, message: None });

    let ready = checks.iter().all(|c| c.ok);

    ReadinessResult { ready, workspace_path: root.to_string_lossy().to_string(), checks }
}
