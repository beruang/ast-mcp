// Rust struct/enum fixtures
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub age: u32,
}

pub struct Post {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author: User,
    pub tags: Vec<String>,
}

pub enum Status {
    Active,
    Inactive,
    Pending { since: String },
}

pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
