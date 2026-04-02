use rand::Rng;

/// Generate a short alphanumeric token for auth challenges (user-typeable).
pub fn generate_challenge_token() -> String {
    let mut rng = rand::rng();
    (0..8)
        .map(|_| {
            let idx = rng.random_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect()
}

/// Generate a long random token for sessions (hex-encoded, 64 chars).
pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
