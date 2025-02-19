use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Write;
use std::net::SocketAddr;
use std::process::Command;
use tempfile::NamedTempFile;
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct FormatRequest {
    code: String,
    max_width: u32,
    indent_size: u32,
}

#[derive(Serialize)]
struct FormatResponse {
    formatted_code: String,
}

async fn format_code(Json(request): Json<FormatRequest>) -> Json<FormatResponse> {
    // Create a temporary file for the code
    let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    write!(temp_file, "{}", request.code).expect("Failed to write to temporary file");
    let temp_file_path = temp_file.path().to_str().unwrap();

    // Create a temporary configuration file
    let config_content = format!(
        "# .afmt.toml - Configuration for afmt\n\n# Maximum line width\nmax_width = {}\n\n# Indentation size in spaces\nindent_size = {}\n",
        request.max_width, request.indent_size
    );
    let mut config_file = NamedTempFile::new().expect("Failed to create temporary config file");
    write!(config_file, "{}", config_content).expect("Failed to write to temporary config file");
    let config_file_path = config_file.path().to_str().unwrap();

    // Run the afmt command with the configuration file
    let output = Command::new("./afmt")
        .arg("--config")
        .arg(config_file_path)
        .arg(temp_file_path)
        .output()
        .expect("Failed to execute command");

    let formatted_code = String::from_utf8(output.stdout).unwrap();

    Json(FormatResponse { formatted_code })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/format", post(format_code))
        .layer(CorsLayer::permissive());

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse().unwrap()));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
