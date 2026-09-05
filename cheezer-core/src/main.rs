mod action;
mod dashboard;
mod executor;
mod fallback;
mod gitops;
mod guard;
mod ingest;
mod llm;
mod policy;
mod state;
mod store;
mod triage;
mod watchdog;
mod devin;
mod predictive;



use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    let mut role = "primary";
    let mut peer = "127.0.0.1:9000";

    for arg in &args {
        if arg.starts_with("--role=") {
            role = arg.split('=').nth(1).unwrap_or("primary");
        } else if arg.starts_with("--peer=") {
            peer = arg.split('=').nth(1).unwrap_or("127.0.0.1:9000");
        }
    }

    log::info!("Starting Cheezer as {}", role);
    store::init_db().expect("Failed to initialize database");

    if role == "backup" {
        watchdog::run_backup(peer).await;
        log::info!("Backup took over! Starting webhook listener...");
    } else {
        // Start primary watchdog listener
        tokio::spawn(async move {
            watchdog::run_primary("0.0.0.0:9000").await;
        });
    }

    let app = ingest::create_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], 9090));
    log::info!("Listening for webhooks on {}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
