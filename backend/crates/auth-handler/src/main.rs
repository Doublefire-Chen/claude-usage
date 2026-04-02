use std::env;
use std::path::PathBuf;

fn main() {
    // Load .env from the same directory as the binary
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let env_path: PathBuf = dir.join(".env");
            let _ = dotenvy::from_path(&env_path);
        }
    }

    // Parse --user from program args (set via authorized_keys command=)
    let args: Vec<String> = env::args().collect();
    let username = args.iter().find_map(|a| a.strip_prefix("--user=")).map(String::from);

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

    let mut body = serde_json::json!({ "token": token });
    if let Some(user) = &username {
        body["username"] = serde_json::json!(user);
    }

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
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
