// Rust test fixtures
#[test]
fn test_simple() {
    assert_eq!(1 + 1, 2);
}

#[test]
fn test_user_get() {
    let user = get_user("1");
    assert_eq!(user.id, "1");
}

#[tokio::test]
async fn test_async_operation() {
    let result = async_call().await;
    assert!(result.is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_module() {
        assert!(true);
    }
}
