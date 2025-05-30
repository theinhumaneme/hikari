use dotenvy::dotenv;

pub fn load_secrets() -> (String, String) {
    dotenv().ok();
    let public_key_path: String = std::env::var("PUBLIC_KEY_FILENAME").expect(
        "PUBLIC_KEY_FILENAME must
    be set.",
    );
    let private_key_path: String = std::env::var("PRIVATE_KEY_FILENAME").expect(
        "PRIVATE_KEY_FILENAME must
    be set.",
    );
    return (public_key_path, private_key_path);
}
