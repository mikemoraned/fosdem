use tracing::info;

pub fn load_secret(name: &str) -> String {
    let secret = dotenvy::var(name).unwrap_or_else(|_| panic!("{} is not set", &name));
    let suffix = secret[(secret.len() - 3)..].to_string();
    info!(
        "Loaded secret with name '{}', ending with '{}'",
        name, suffix
    );
    secret
}
