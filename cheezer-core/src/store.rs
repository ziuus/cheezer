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


