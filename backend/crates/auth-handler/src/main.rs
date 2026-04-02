use std::env;

fn main() {
    let command = env::var("SSH_ORIGINAL_COMMAND").unwrap_or_default();

    let token = match parse_token(&command) {
        Some(t) => t,
        None => {
            eprintln!("Usage: ssh claude-auth@host auth=<token>");
            std::process::exit(1);
        }
    };

    let server_url =
        env::var("AUTH_SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    let url = format!("{server_url}/internal/auth/verify");

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "token": token }))
        .send();

    match resp {
        Ok(r) if r.status().is_success() => {
            println!("Authenticated successfully. You can close this terminal.");
        }
        Ok(r) => {
            eprintln!("Authentication failed: {}", r.status());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to contact auth server: {e}");
            std::process::exit(1);
        }
    }
}

fn parse_token(command: &str) -> Option<&str> {
    command
        .split_whitespace()
        .find_map(|part| part.strip_prefix("auth="))
}
