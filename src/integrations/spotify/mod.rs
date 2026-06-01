pub mod oauth;

pub enum AuthResult {
    Success { username: String },
    Failure(String),
}
