use dotenvy::dotenv;
use log::warn;

use super::error::ConfigError;

pub fn load_secrets(mode: &str) -> Result<Vec<String>, ConfigError> {
    dotenv().ok();
    Ok(match mode {
        "daemon" => {
            let public_key_path: String = std::env::var("PUBLIC_KEY_FILENAME")
                .map_err(|_| ConfigError::MissingField("PUBLIC_KEY_FILENAME".into()))?;
            let private_key_path: String = std::env::var("PRIVATE_KEY_FILENAME")
                .map_err(|_| ConfigError::MissingField("PRIVATE_KEY_FILENAME".into()))?;
            vec![public_key_path, private_key_path]
        }
        "server" => {
            let pg_host: String = std::env::var("POSTGRES_HOST")
                .map_err(|_| ConfigError::MissingField("POSTGRES_HOST".into()))?;
            let pg_db: String = std::env::var("POSTGRES_DATABASE")
                .map_err(|_| ConfigError::MissingField("POSTGRES_DATABASE".into()))?;
            let pg_user: String = std::env::var("POSTGRES_USER")
                .map_err(|_| ConfigError::MissingField("POSTGRES_USER".into()))?;
            let pg_pass: String = std::env::var("POSTGRES_PASSWORD")
                .map_err(|_| ConfigError::MissingField("POSTGRES_PASSWORD".into()))?;
            let pg_port: String = std::env::var("POSTGRES_PORT")
                .map_err(|_| ConfigError::MissingField("POSTGRES_PORT".into()))?;
            vec![pg_user, pg_pass, pg_host, pg_port, pg_db]
        }
        "agent" => {
            let hikari_server: String = std::env::var("HIKARI_SERVER_DOMAIN")
                .map_err(|_| ConfigError::MissingField("HIKARI_SERVER_DOMAIN".into()))?;
            vec![hikari_server]
        }

        _ => {
            warn!("Secrets Could Not be loaded as no MODE configured");
            vec![]
        }
    })
}
