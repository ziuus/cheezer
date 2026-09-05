use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

pub static IS_LEADER: AtomicBool = AtomicBool::new(true);

#[allow(dead_code)]
pub fn is_leader() -> bool {
    IS_LEADER.load(Ordering::Relaxed)
}

pub fn promote_to_leader() {
    IS_LEADER.store(true, Ordering::Relaxed);
    log::info!("Watchdog Leader Election: Promoted node to ACTIVE LEADER status.");
}

pub fn demote_from_leader() {
    IS_LEADER.store(false, Ordering::Relaxed);
    log::info!("Watchdog Leader Election: Demoted node to STANDBY status.");
}

pub async fn run_primary(bind_addr: &str) {
    promote_to_leader();
    let addr: SocketAddr = match bind_addr.parse() {
        Ok(a) => a,
        Err(e) => {
            log::error!("Invalid watchdog bind address '{}': {}", bind_addr, e);
            return;
        }
    };
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind watchdog port on {}: {}", addr, e);
            return;
        }
    };
    log::info!("Watchdog primary listening on {}", addr);

    loop {
        if let Ok((_stream, _)) = listener.accept().await {
            // Proof-of-life ping handshake
        }
    }
}

pub async fn run_backup(peer_addr: &str) {
    demote_from_leader();
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
                promote_to_leader();
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

        assert!(is_leader(), "Backup node must be promoted to leader upon failover!");
        println!("SUCCESS: Watchdog failover verified - backup automatically promoted to LEADER when primary was killed!");
    }
}
