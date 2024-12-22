use tracing::info;

pub fn load_secret(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let secret = dotenvy::var(name).map_err(|e| format!("{} is not set, e: {:?}", &name, e))?;
    let suffix = secret[(secret.len() - 3)..].to_string();
    info!(
        "Loaded secret with name '{}', ending with '{}'",
        name, suffix
    );
    Ok(secret)
}

pub fn load_public(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let value = dotenvy::var(name).map_err(|e| format!("{} is not set, e: {:?}", &name, e))?;
    info!("Loaded public item with name '{}', value '{}'", name, value);
    Ok(value)
}
