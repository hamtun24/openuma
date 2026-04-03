use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub id: Option<i64>,
    pub name: String,
    pub cpu_model: String,
    pub igpu: Option<String>,
    pub ram_mb: u64,
    pub os: String,
    pub created_at: Option<String>,
}

pub struct ProfileDatabase {
    conn: Connection,
}

impl ProfileDatabase {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                cpu_model TEXT NOT NULL,
                igpu TEXT,
                ram_mb INTEGER NOT NULL,
                os TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn insert_profile(&self, profile: &HardwareProfile) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO profiles (name, cpu_model, igpu, ram_mb, os) VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &profile.name,
                &profile.cpu_model,
                &profile.igpu,
                &profile.ram_mb,
                &profile.os,
            ),
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_profiles(&self) -> Result<Vec<HardwareProfile>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, cpu_model, igpu, ram_mb, os, created_at FROM profiles")?;
        let profiles = stmt
            .query_map([], |row| {
                Ok(HardwareProfile {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    cpu_model: row.get(2)?,
                    igpu: row.get(3)?,
                    ram_mb: row.get(4)?,
                    os: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(profiles)
    }

    pub fn find_profile(&self, cpu_model: &str, _ram_mb: u64) -> Result<Option<HardwareProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, cpu_model, igpu, ram_mb, os, created_at FROM profiles WHERE cpu_model LIKE ?1 LIMIT 1"
        )?;
        let mut rows = stmt.query([cpu_model])?;
        if let Some(row) = rows.next()? {
            Ok(Some(HardwareProfile {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                cpu_model: row.get(2)?,
                igpu: row.get(3)?,
                ram_mb: row.get(4)?,
                os: row.get(5)?,
                created_at: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }
}
