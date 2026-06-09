// Rust attribute fixtures
#[derive(Debug, Clone)]
struct User {
    id: String,
    name: String,
}

#[test]
fn test_user() {
    assert!(true);
}

#[tokio::main]
async fn main() {
    println!("Hello");
}

#[cfg(test)]
mod tests {
    #[test]
    fn inner_test() {}
}
