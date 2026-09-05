use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentRecord {
    pub id: i64,
    pub signature: String,
    pub severity: String,
    pub mode: String,
    pub action: String,
    pub status: String,
    pub verification_result: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemediationRecord {
    pub id: i64,
    pub incident_id: i64,
    pub resource: String,
    pub action: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoredTarget {
    pub id: i64,
    pub name: String,
    pub provider: String,
    pub external_id: String,
    pub environment: String,
    pub github_repo: String,
    pub custom_instructions: String,
    pub status: String,
    pub created_at: String,
}

fn get_db() -> &'static Mutex<Connection> {
    static DB: OnceLock<Mutex<Connection>> = OnceLock::new();
    DB.get_or_init(|| {
        let conn = Connection::open("cheezer.db").expect("Failed to open DB");
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "busy_timeout", "5000").unwrap();
        Mutex::new(conn)
    })
}

pub fn init_db() -> Result<()> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS incidents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            signature TEXT NOT NULL,
            severity TEXT,
            mode TEXT NOT NULL,
            action TEXT NOT NULL,
            status TEXT NOT NULL,
            verification_result TEXT DEFAULT 'N/A',
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    let _ = conn.execute("ALTER TABLE incidents ADD COLUMN verification_result TEXT DEFAULT 'N/A'", []);
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS alerts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            signature TEXT NOT NULL,
            severity TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS signature_history (
            signature TEXT PRIMARY KEY,
            first_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
            occurrence_count INTEGER DEFAULT 1,
            self_resolved_count INTEGER DEFAULT 0
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS action_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            alert_id INTEGER,
            mode TEXT,
            action TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS remediation_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            incident_id INTEGER NOT NULL,
            resource TEXT NOT NULL,
            action TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS credentials (
            service TEXT PRIMARY KEY,
            token TEXT NOT NULL,
            endpoint TEXT,
            status TEXT DEFAULT 'CONFIGURED',
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS monitored_targets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            provider TEXT NOT NULL,
            external_id TEXT NOT NULL,
            environment TEXT DEFAULT 'production',
            github_repo TEXT,
            custom_instructions TEXT,
            status TEXT DEFAULT 'WATCHING',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    seed_default_data(&conn)?;

    Ok(())
}

fn seed_default_data(conn: &Connection) -> Result<()> {
    let incident_count: i64 = conn.query_row("SELECT COUNT(*) FROM incidents", [], |row| row.get(0)).unwrap_or(0);
    if incident_count == 0 {
        conn.execute(
            "INSERT INTO incidents (signature, severity, mode, action, status, verification_result, timestamp) VALUES 
            ('CrashLoopBackOff', 'critical', 'rule', 'restart deployment flaky-order-service', 'executed', 'TOCTOU check passed. Target status returned HTTP 200 OK.', '2026-09-05 15:30:10'),
            ('HighMemoryUtilization', 'warning', 'ai', 'scale deployment floci-order-processor --replicas=3', 'executed', 'Resource utilization normalized to 45%.', '2026-09-05 16:12:45'),
            ('UnauthorizedDeleteNamespace', 'critical', 'rule', 'delete namespace production', 'blocked_by_opa', 'OPA Policy Engine Gate: Action DENIED (Fail-Closed).', '2026-09-05 16:45:00'),
            ('CascadingDatabaseTimeout', 'critical', 'ai', 'restart deployment billing-api-service', 'requires_human_intervention', 'Circuit Breaker Engaged: Awaiting Operator Approval.', '2026-09-05 17:10:30'),
            ('VercelEdgeDeploymentError', 'warning', 'rule', 'rollback vercel deployment prj_storefront992', 'executed', 'Vercel edge active. HTTP 200 OK.', '2026-09-05 17:28:15')",
            [],
        )?;

        conn.execute(
            "INSERT INTO remediation_history (incident_id, resource, action, timestamp) VALUES 
            (1, 'flaky-order-service', 'restart deployment flaky-order-service', '2026-09-05 15:30:12'),
            (2, 'floci-order-processor', 'scale deployment floci-order-processor --replicas=3', '2026-09-05 16:12:48'),
            (5, 'production-storefront', 'rollback vercel deployment prj_storefront992', '2026-09-05 17:28:18')",
            [],
        )?;
    }

    let watcher_count: i64 = conn.query_row("SELECT COUNT(*) FROM monitored_targets", [], |row| row.get(0)).unwrap_or(0);
    if watcher_count == 0 {
        conn.execute(
            "INSERT INTO monitored_targets (name, provider, external_id, environment, github_repo, custom_instructions, status) VALUES 
            ('flaky-order-service (Deployment)', 'k8s', 'flaky-order-service', 'demo', 'ziuus/order-microservice', 'Restart deployment on CrashLoopBackOff & 5xx HTTP spikes', 'WATCHING'),
            ('cheezer-core (Control Plane)', 'k8s', 'cheezer-core', 'demo', 'ziuus/cheezer', 'Self-remediate OPA policy violations and monitor system health', 'WATCHING'),
            ('production-storefront (Vercel Web)', 'vercel', 'prj_storefront992', 'production', 'ziuus/storefront', 'Rollback Vercel deployment on 502/504 edge errors', 'WATCHING'),
            ('floci-order-processor (AWS)', 'aws', 'floci-order-processor', 'us-east-1', 'ziuus/order-processor', 'Autoscale SQS workers and clear stuck queues', 'WATCHING'),
            ('billing-api-service (Cloud Run)', 'gcloud', 'billing-api-service', 'us-central1', 'ziuus/billing-api', 'Redeploy Cloud Run revision on memory leak detection', 'WATCHING')",
            [],
        )?;
    }

    Ok(())
}

pub fn clear_db() -> Result<()> {
    let conn = get_db().lock().unwrap();
    conn.execute("DELETE FROM incidents", [])?;
    conn.execute("DELETE FROM alerts", [])?;
    conn.execute("DELETE FROM action_log", [])?;
    conn.execute("DELETE FROM signature_history", [])?;
    conn.execute("DELETE FROM remediation_history", [])?;
    Ok(())
}

pub fn log_remediation(incident_id: i64, resource: &str, action: &str) -> Result<i64> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "INSERT INTO remediation_history (incident_id, resource, action) VALUES (?1, ?2, ?3)",
        rusqlite::params![incident_id, resource, action],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn reset_resource_remediations(resource: &str) -> Result<()> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "DELETE FROM remediation_history WHERE resource = ?1",
        rusqlite::params![resource],
    )?;
    Ok(())
}

pub fn get_remediations() -> Result<Vec<RemediationRecord>> {
    let conn = get_db().lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, incident_id, resource, action, DATETIME(timestamp) FROM remediation_history ORDER BY id DESC LIMIT 50")?;
    let rows = stmt.query_map([], |row| {
        Ok(RemediationRecord {
            id: row.get(0)?,
            incident_id: row.get(1)?,
            resource: row.get(2)?,
            action: row.get(3)?,
            timestamp: row.get(4).unwrap_or_default(),
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn get_resource_action_count(resource: &str, window_seconds: i64) -> Result<i64> {
    let conn = get_db().lock().unwrap();
    let query = format!(
        "SELECT COUNT(*) FROM remediation_history WHERE resource = ?1 AND timestamp >= DATETIME('now', '-{} seconds')",
        window_seconds
    );
    let count: i64 = conn.query_row(&query, rusqlite::params![resource], |r| r.get(0))?;
    Ok(count)
}

pub fn get_incident_action_count(incident_id: i64) -> Result<i64> {
    let conn = get_db().lock().unwrap();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM remediation_history WHERE incident_id = ?1",
        rusqlite::params![incident_id],
        |r| r.get(0)
    )?;
    Ok(count)
}

pub fn get_seconds_since_last_resource_action(resource: &str) -> Result<Option<i64>> {
    let conn = get_db().lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT CAST((JULIANDAY('now') - JULIANDAY(timestamp)) * 86400 AS INTEGER) FROM remediation_history WHERE resource = ?1 ORDER BY id DESC LIMIT 1"
    )?;
    let mut rows = stmt.query(rusqlite::params![resource])?;
    if let Some(row) = rows.next()? {
        let secs: i64 = row.get(0)?;
        Ok(Some(secs))
    } else {
        Ok(None)
    }
}

pub fn log_incident(signature: &str, severity: &str, mode: &str, action: &str, status: &str) -> Result<i64> {
    log_incident_with_verification(signature, severity, mode, action, status, "N/A")
}

pub fn log_incident_with_verification(
    signature: &str,
    severity: &str,
    mode: &str,
    action: &str,
    status: &str,
    verification: &str,
) -> Result<i64> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "INSERT INTO incidents (signature, severity, mode, action, status, verification_result) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![signature, severity, mode, action, status, verification],
    )?;
    
    let incident_id = conn.last_insert_rowid();

    let updated = conn.execute(
        "UPDATE signature_history 
         SET last_seen = CURRENT_TIMESTAMP, occurrence_count = occurrence_count + 1 
         WHERE signature = ?1",
        rusqlite::params![signature],
    )?;
    
    if updated == 0 {
        conn.execute(
            "INSERT INTO signature_history (signature) VALUES (?1)",
            rusqlite::params![signature],
        )?;
    }

    Ok(incident_id)
}

pub fn update_incident_status(id: i64, status: &str) -> Result<()> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "UPDATE incidents SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, id],
    )?;
    Ok(())
}

pub fn update_incident_verification(id: i64, verification: &str) -> Result<()> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "UPDATE incidents SET verification_result = ?1 WHERE id = ?2",
        rusqlite::params![verification, id],
    )?;
    Ok(())
}

pub fn get_incident_by_id(id: i64) -> Result<Option<IncidentRecord>> {
    let conn = get_db().lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, signature, COALESCE(severity, ''), mode, action, status, COALESCE(verification_result, 'N/A'), DATETIME(timestamp) FROM incidents WHERE id = ?1")?;
    let mut rows = stmt.query(rusqlite::params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(IncidentRecord {
            id: row.get(0)?,
            signature: row.get(1)?,
            severity: row.get(2)?,
            mode: row.get(3)?,
            action: row.get(4)?,
            status: row.get(5)?,
            verification_result: row.get(6)?,
            timestamp: row.get(7).unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

pub fn get_incidents() -> Result<Vec<IncidentRecord>> {
    let conn = get_db().lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, signature, COALESCE(severity, ''), mode, action, status, COALESCE(verification_result, 'N/A'), DATETIME(timestamp) FROM incidents ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(IncidentRecord {
            id: row.get(0)?,
            signature: row.get(1)?,
            severity: row.get(2)?,
            mode: row.get(3)?,
            action: row.get(4)?,
            status: row.get(5)?,
            verification_result: row.get(6)?,
            timestamp: row.get(7).unwrap_or_default(),
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn log_alert(signature: &str, severity: &str) -> Result<i64> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "INSERT INTO alerts (signature, severity) VALUES (?1, ?2)",
        rusqlite::params![signature, severity],
    )?;
    
    let alert_id = conn.last_insert_rowid();
    
    let updated = conn.execute(
        "UPDATE signature_history 
         SET last_seen = CURRENT_TIMESTAMP, occurrence_count = occurrence_count + 1 
         WHERE signature = ?1",
        rusqlite::params![signature],
    )?;
    
    if updated == 0 {
        conn.execute(
            "INSERT INTO signature_history (signature) VALUES (?1)",
            rusqlite::params![signature],
        )?;
    }
    
    Ok(alert_id)
}

pub fn get_signature_stats(signature: &str) -> Result<(i64, i64)> {
    let conn = get_db().lock().unwrap();
    let mut stmt = conn.prepare("SELECT occurrence_count, self_resolved_count FROM signature_history WHERE signature = ?1")?;
    let mut rows = stmt.query(rusqlite::params![signature])?;
    if let Some(row) = rows.next()? {
        Ok((row.get(0)?, row.get(1)?))
    } else {
        Ok((0, 0))
    }
}

pub fn log_action(alert_id: i64, mode: &str, action: &str) -> Result<()> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "INSERT INTO action_log (alert_id, mode, action) VALUES (?1, ?2, ?3)",
        rusqlite::params![alert_id, mode, action],
    )?;
    Ok(())
}

pub fn save_credential(service: &str, token: &str, endpoint: &str, status: &str) -> Result<()> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "INSERT INTO credentials (service, token, endpoint, status, updated_at)
         VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
         ON CONFLICT(service) DO UPDATE SET
         token=excluded.token, endpoint=excluded.endpoint, status=excluded.status, updated_at=CURRENT_TIMESTAMP",
        rusqlite::params![service, token, endpoint, status],
    )?;
    Ok(())
}

pub fn get_credential(service: &str) -> Result<Option<(String, String, String)>> {
    let conn = get_db().lock().unwrap();
    let mut stmt = conn.prepare("SELECT token, COALESCE(endpoint, ''), COALESCE(status, 'UNCONFIGURED') FROM credentials WHERE service = ?1")?;
    let mut rows = stmt.query(rusqlite::params![service])?;
    if let Some(row) = rows.next()? {
        Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?)))
    } else {
        Ok(None)
    }
}

pub fn create_monitored_target(
    name: &str,
    provider: &str,
    external_id: &str,
    environment: &str,
    github_repo: &str,
    custom_instructions: &str,
) -> Result<i64> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "INSERT INTO monitored_targets (name, provider, external_id, environment, github_repo, custom_instructions, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'WATCHING')",
        rusqlite::params![name, provider, external_id, environment, github_repo, custom_instructions],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_monitored_targets() -> Result<Vec<MonitoredTarget>> {
    let conn = get_db().lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, provider, external_id, COALESCE(environment, 'production'), COALESCE(github_repo, ''), COALESCE(custom_instructions, ''), status, DATETIME(created_at)
         FROM monitored_targets ORDER BY id DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(MonitoredTarget {
            id: row.get(0)?,
            name: row.get(1)?,
            provider: row.get(2)?,
            external_id: row.get(3)?,
            environment: row.get(4)?,
            github_repo: row.get(5)?,
            custom_instructions: row.get(6)?,
            status: row.get(7)?,
            created_at: row.get(8).unwrap_or_default(),
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn delete_monitored_target(id: i64) -> Result<()> {
    let conn = get_db().lock().unwrap();
    conn.execute("DELETE FROM monitored_targets WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}


