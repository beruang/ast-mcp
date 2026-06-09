use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools::find_routes;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

fn call(file_path: &str) -> serde_json::Value {
    find_routes::handle(&workspace(), json!({"file_path": file_path}))
}

fn assert_no_error(result: &serde_json::Value) {
    if result.get("error").is_some() {
        panic!("unexpected error: {:?}", result["error"]);
    }
}

#[test]
fn routes_express_detects_all_methods() {
    let r = call("tests/fixtures/v3/routes/express.ts");
    assert_no_error(&r);
    let routes = r["routes"].as_array().unwrap();
    assert!(routes.len() >= 7, "expected at least 7 routes, got {}", routes.len());
    let methods: Vec<&str> = routes.iter().filter_map(|rt| rt["method"].as_str()).collect();
    assert!(methods.contains(&"get"));
    assert!(methods.contains(&"post"));
    assert!(methods.contains(&"put"));
    assert!(methods.contains(&"delete"));
}

#[test]
fn routes_express_has_path_and_handler() {
    let r = call("tests/fixtures/v3/routes/express.ts");
    assert_no_error(&r);
    let routes = r["routes"].as_array().unwrap();
    let get_users = routes
        .iter()
        .find(|rt| rt["path"].as_str() == Some("/users") && rt["method"].as_str() == Some("get"));
    assert!(get_users.is_some(), "should find GET /users route");

    for rt in routes {
        assert!(rt["confidence"].as_str().is_some());
        assert!(rt["evidence"].as_array().is_some_and(|a| !a.is_empty()));
    }
}

#[test]
fn routes_nextjs_detects_exported_handlers() {
    let r = call("tests/fixtures/v3/routes/nextjs.ts");
    assert_no_error(&r);
    let routes = r["routes"].as_array().unwrap();
    assert_eq!(routes.len(), 4);
    let methods: Vec<&str> = routes.iter().filter_map(|rt| rt["method"].as_str()).collect();
    assert!(methods.contains(&"get"));
    assert!(methods.contains(&"post"));
    assert!(methods.contains(&"put"));
    assert!(methods.contains(&"delete"));
}

#[test]
fn routes_nestjs_detects_controller_and_methods() {
    let r = call("tests/fixtures/v3/routes/nestjs.ts");
    assert_no_error(&r);
    let routes = r["routes"].as_array().unwrap();
    assert!(routes.len() >= 5);
    for rt in routes {
        let framework = rt["framework"].as_str().unwrap();
        assert_eq!(framework, "nestjs");
    }
}

#[test]
fn routes_fastapi_detects_decorator_routes() {
    let r = call("tests/fixtures/v3/routes/fastapi.py");
    assert_no_error(&r);
    let routes = r["routes"].as_array().unwrap();
    assert!(routes.len() >= 4);
    let methods: Vec<&str> = routes.iter().filter_map(|rt| rt["method"].as_str()).collect();
    assert!(methods.contains(&"get"));
    assert!(methods.contains(&"post"));
}

#[test]
fn routes_flask_detects_route_decorators() {
    let r = call("tests/fixtures/v3/routes/flask.py");
    assert_no_error(&r);
    let routes = r["routes"].as_array().unwrap();
    assert!(routes.len() >= 3);
    // Should have GET /users
    let get_users = routes
        .iter()
        .find(|rt| rt["path"].as_str() == Some("/users") && rt["method"].as_str() == Some("get"));
    assert!(get_users.is_some(), "should find GET /users");
}

#[test]
fn routes_framework_filter_works() {
    let r = find_routes::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/routes/express.ts",
            "frameworks": ["express"]
        }),
    );
    assert_no_error(&r);
    let routes = r["routes"].as_array().unwrap();
    for rt in routes {
        let fw = rt["framework"].as_str().unwrap();
        assert!(fw.contains("express"), "expected express framework, got {}", fw);
    }
}

#[test]
fn routes_has_confidence_and_evidence() {
    let r = call("tests/fixtures/v3/routes/express.ts");
    assert_no_error(&r);
    for rt in r["routes"].as_array().unwrap() {
        let conf = rt["confidence"].as_str().expect("missing confidence");
        assert!(conf == "high" || conf == "medium" || conf == "low");
        let evidence = rt["evidence"].as_array().expect("missing evidence");
        assert!(!evidence.is_empty(), "evidence should not be empty");
    }
}
