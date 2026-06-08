use axum::{
    routing::post,
    Router,
    Json,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use tower_http::services::{ServeDir, ServeFile};
use std::convert::Infallible;
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio_stream::wrappers::ReceiverStream;
use futures_util::stream::StreamExt;

mod agent;

struct AppState {
    client: reqwest::Client,
    env_api_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize agent uptime tracker
    agent::init_uptime();

    // Read optional Gemini API Key from host environment
    let env_api_key = std::env::var("GEMINI_API_KEY").ok();
    if env_api_key.is_some() {
        tracing::info!("Found GEMINI_API_KEY in server environment.");
    } else {
        tracing::info!("No GEMINI_API_KEY found in server environment. Starting in Demo Mode by default.");
    }
    
    let state = Arc::new(AppState {
        client: reqwest::Client::new(),
        env_api_key,
    });

    // Permissive CORS for easy development
    let cors = CorsLayer::permissive();

    // Serve static files from './static' directory, falling back to 'index.html' for SPA routing
    let serve_dir = ServeDir::new("static")
        .fallback(ServeFile::new("static/index.html"));

    let app = Router::new()
        .route("/api/chat", post(chat_handler))
        .fallback_service(serve_dir)
        .layer(cors)
        .with_state(state);

    let port = 3000;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Server running at http://localhost:{}", port);
    tracing::info!("🚀 Server running at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// SSE streaming endpoint — sends events as the agent processes
async fn chat_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(payload): Json<agent::ChatRequest>,
) -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::channel::<agent::SseEvent>(64);

    let client = state.client.clone();
    let env_key = state.env_api_key.clone();

    // Spawn the agent loop in background — it sends events through the channel
    tokio::spawn(async move {
        agent::run_agent_loop_streaming(payload, &client, env_key, tx).await;
    });

    // Convert the mpsc receiver into an SSE stream
    let stream = ReceiverStream::new(rx).map(|event| -> Result<Event, Infallible> {
        let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
        Ok(Event::default().data(json))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
