use dotenvy::dotenv;
use log::warn;

pub fn load_secrets(mode: &str) -> Vec<String> {
    dotenv().ok();
    match mode {
        "daemon" => {
            let public_key_path: String = std::env::var("PUBLIC_KEY_FILENAME").expect(
                "PUBLIC_KEY_FILENAME must
            be set.",
            );
            let private_key_path: String = std::env::var("PRIVATE_KEY_FILENAME").expect(
                "PRIVATE_KEY_FILENAME must
            be set.",
            );
            vec![public_key_path, private_key_path]
        }
        "server" => {
            let pg_host: String = std::env::var("POSTGRES_HOST").expect(
                "POSTGRES_HOST must
            be set.",
            );
            let pg_db: String = std::env::var("POSTGRES_DATABASE").expect(
                "POSTGRES_DATABASE must
            be set.",
            );
            let pg_user: String = std::env::var("POSTGRES_USER").expect(
                "POSTGRES_USER must
            be set.",
            );
            let pg_pass: String = std::env::var("POSTGRES_PASSWORD").expect(
                "POSTGRES_PASSWORD must
            be set.",
            );
            let pg_port: String = std::env::var("POSTGRES_PORT").expect(
                "POSTGRES_PORT must
            be set.",
            );
            vec![pg_user, pg_pass, pg_host, pg_port, pg_db]
        }
        _ => {
            warn!("Secrets Could Not be loaded as no MODE configured");
            vec![]
        }
    }
}
