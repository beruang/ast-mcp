use crate::analysis::duplicate_shapes;
use serde_json::Value;

pub fn handle(arguments: Value) -> Value {
    let _input = duplicate_shapes::handle_arguments(arguments);
    let result = duplicate_shapes::empty_result(0);
    serde_json::to_value(result).unwrap_or_default()
}
