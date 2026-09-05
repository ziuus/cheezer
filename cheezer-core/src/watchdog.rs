use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

pub async fn run_primary(bind_addr: &str) {
    let addr: SocketAddr = bind_addr.parse().unwrap();
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind watchdog port");
    log::info!("Watchdog primary listening on {}", addr);

    loop {
        if let Ok((_stream, _)) = listener.accept().await {
            // Just accept and drop the connection as proof-of-life
        }
    }
}

pub async fn run_backup(peer_addr: &str) {
    run_backup_interval(peer_addr, Duration::from_secs(2)).await;
}

pub async fn run_backup_interval(peer_addr: &str, poll_interval: Duration) {
    log::info!("Watchdog backup monitoring {}", peer_addr);
    loop {
        match TcpStream::connect(peer_addr).await {
            Ok(_) => {
                sleep(poll_interval).await;
            }
            Err(_) => {
                log::warn!("Primary watchdog at {} is dead! Backup taking over.", peer_addr);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn test_watchdog_failover() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let primary_handle = tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx => {},
                _ = async {
                    loop {
                        if let Ok((_stream, _)) = listener.accept().await {}
                    }
                } => {}
            }
        });

        let peer = addr.clone();
        let backup_handle = tokio::spawn(async move {
            run_backup_interval(&peer, Duration::from_millis(50)).await;
        });

        sleep(Duration::from_millis(100)).await;
        assert!(!backup_handle.is_finished(), "Backup must remain monitoring while primary is alive!");

        let _ = shutdown_tx.send(());
        let _ = primary_handle.await;

        tokio::time::timeout(Duration::from_secs(2), backup_handle)
            .await
            .expect("Backup must detect primary death within timeout")
            .unwrap();

        println!("SUCCESS: Watchdog failover verified - backup automatically took over when primary was killed!");
    }
}

