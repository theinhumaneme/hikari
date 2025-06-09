use reqwest::StatusCode;
use sqlx::Error;

pub fn map_db_error(e: Error) -> (StatusCode, String) {
    match e {
        Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                let c = db_err.constraint().unwrap_or("unknown");
                return (
                    StatusCode::CONFLICT,
                    format!("Duplicate entry: `{}` constraint", c),
                );
            }
            if db_err.is_foreign_key_violation() {
                let c = db_err.constraint().unwrap_or("unknown");
                return (
                    StatusCode::CONFLICT,
                    format!("Foreign key violation: `{}` constraint", c),
                );
            }
            if db_err.is_check_violation() {
                let c = db_err.constraint().unwrap_or("unknown");
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Check violation: `{}` constraint", c),
                );
            }
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into());
        }
        Error::RowNotFound => (StatusCode::NOT_FOUND, "Record not found".into()),

        Error::Io(err) => (StatusCode::SERVICE_UNAVAILABLE, err.to_string()),

        Error::Protocol(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),

        Error::Tls(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),

        Error::PoolTimedOut => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Connection timed out".into(),
        ),

        Error::TypeNotFound { type_name } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Type not found: {}", type_name),
        ),
        Error::ColumnNotFound(col) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Column not found: {}", col),
        ),
        Error::ColumnIndexOutOfBounds { index, len } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Column index out of bounds: {}/{}", index, len),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL SERVER ERROR".into(),
        ),
    }
}
