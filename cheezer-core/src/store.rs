use rusqlite::{Connection, Result};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq)]
pub struct IncidentRecord {
    pub id: i64,
    pub signature: String,
    pub severity: String,
    pub mode: String,
    pub action: String,
    pub status: String,
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
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
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
    
    Ok(())
}

pub fn clear_db() -> Result<()> {
    let conn = get_db().lock().unwrap();
    conn.execute("DELETE FROM incidents", [])?;
    conn.execute("DELETE FROM alerts", [])?;
    conn.execute("DELETE FROM action_log", [])?;
    conn.execute("DELETE FROM signature_history", [])?;
    Ok(())
}

pub fn log_incident(signature: &str, severity: &str, mode: &str, action: &str, status: &str) -> Result<i64> {
    let conn = get_db().lock().unwrap();
    conn.execute(
        "INSERT INTO incidents (signature, severity, mode, action, status) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![signature, severity, mode, action, status],
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

pub fn get_incidents() -> Result<Vec<IncidentRecord>> {
    let conn = get_db().lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, signature, COALESCE(severity, ''), mode, action, status, DATETIME(timestamp) FROM incidents ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(IncidentRecord {
            id: row.get(0)?,
            signature: row.get(1)?,
            severity: row.get(2)?,
            mode: row.get(3)?,
            action: row.get(4)?,
            status: row.get(5)?,
            timestamp: row.get(6).unwrap_or_default(),
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

