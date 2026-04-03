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

const SEED_PROFILES: &[(&str, &str, &str, u64)] = &[
    ("Ryzen 5 5600G", "AMD Ryzen 5 5600G", "AMD Vega7", 32768),
    ("Ryzen 7 5700G", "AMD Ryzen 7 5700G", "AMD Vega8", 65536),
    ("Ryzen 5 5625U", "AMD Ryzen 5 5625U", "AMD Vega7", 16384),
    ("Ryzen 7 5825U", "AMD Ryzen 7 5825U", "AMD Vega8", 32768),
    (
        "Ryzen 5 7535U",
        "AMD Ryzen 5 7535U",
        "AMD Radeon 660M",
        32768,
    ),
    (
        "Ryzen 7 7735U",
        "AMD Ryzen 7 7735U",
        "AMD Radeon 680M",
        32768,
    ),
    (
        "Ryzen 5 7640U",
        "AMD Ryzen 5 7640U",
        "AMD Radeon 760M",
        32768,
    ),
    (
        "Ryzen 7 7840U",
        "AMD Ryzen 7 7840U",
        "AMD Radeon 780M",
        32768,
    ),
    (
        "Ryzen AI 9 HX 370",
        "AMD Ryzen AI 9 HX 370",
        "AMD Radeon 890M",
        65536,
    ),
    (
        "Ryzen AI 7 350",
        "AMD Ryzen AI 7 350",
        "AMD Radeon 880M",
        32768,
    ),
    (
        "Core i5-1240P",
        "12th Gen Intel Core i5-1240P",
        "Intel Iris Xe",
        32768,
    ),
    (
        "Core i7-1260P",
        "12th Gen Intel Core i7-1260P",
        "Intel Iris Xe",
        32768,
    ),
    (
        "Core i5-1250P",
        "12th Gen Intel Core i5-1250P",
        "Intel Iris Xe",
        32768,
    ),
    (
        "Core i7-12700H",
        "12th Gen Intel Core i7-12700H",
        "Intel Iris Xe",
        32768,
    ),
    (
        "Core i5-1340P",
        "13th Gen Intel Core i5-1340P",
        "Intel Iris Xe",
        32768,
    ),
    (
        "Core i7-1360P",
        "13th Gen Intel Core i7-1360P",
        "Intel Iris Xe",
        65536,
    ),
    (
        "Core i5-13500H",
        "13th Gen Intel Core i5-13500H",
        "Intel Iris Xe",
        32768,
    ),
    (
        "Core i7-13700H",
        "13th Gen Intel Core i7-13700H",
        "Intel Iris Xe",
        32768,
    ),
    (
        "Core Ultra 5 125H",
        "Intel Core Ultra 5 125H",
        "Intel Arc",
        32768,
    ),
    (
        "Core Ultra 7 155H",
        "Intel Core Ultra 7 155H",
        "Intel Arc",
        65536,
    ),
    (
        "Core Ultra 7 158V",
        "Intel Core Ultra 7 158V",
        "Intel Arc",
        32768,
    ),
    (
        "Core Ultra 5 228V",
        "Intel Core Ultra 5 228V",
        "Intel Arc",
        16384,
    ),
    (
        "Core i5-12450H",
        "12th Gen Intel Core i5-12450H",
        "Intel UHD",
        16384,
    ),
    (
        "Core i3-1215U",
        "12th Gen Intel Core i3-1215U",
        "Intel UHD",
        16384,
    ),
    (
        "Athlon Silver 7120U",
        "AMD Athlon Silver 7120U",
        "AMD Radeon 610M",
        16384,
    ),
    (
        "Ryzen 3 7320U",
        "AMD Ryzen 3 7320U",
        "AMD Radeon 610M",
        16384,
    ),
    (
        "Ryzen 5 7520U",
        "AMD Ryzen 5 7520U",
        "AMD Radeon 610M",
        16384,
    ),
    (
        "Ryzen 3 8430U",
        "AMD Ryzen 3 8430U",
        "AMD Radeon 740M",
        16384,
    ),
    (
        "Ryzen 5 7530U",
        "AMD Ryzen 5 7530U",
        "AMD Radeon Graphics",
        32768,
    ),
    (
        "Ryzen 7 7730U",
        "AMD Ryzen 7 7730U",
        "AMD Radeon Graphics",
        32768,
    ),
    (
        "Ryzen 5 7540U",
        "AMD Ryzen 5 7540U",
        "AMD Radeon 740M",
        32768,
    ),
    (
        "Ryzen 7 7840HS",
        "AMD Ryzen 7 7840HS",
        "AMD Radeon 780M",
        32768,
    ),
    (
        "Ryzen 9 7940HS",
        "AMD Ryzen 9 7940HS",
        "AMD Radeon 780M",
        65536,
    ),
    (
        "Core i9-12900HK",
        "12th Gen Intel Core i9-12900HK",
        "Intel Iris Xe",
        65536,
    ),
    (
        "Core i7-12800H",
        "12th Gen Intel Core i7-12800H",
        "Intel Iris Xe",
        32768,
    ),
    (
        "Core i5-12600H",
        "12th Gen Intel Core i5-12600H",
        "Intel Iris Xe",
        32768,
    ),
    (
        "Core i9-13900HK",
        "13th Gen Intel Core i9-13900HK",
        "Intel Iris Xe",
        65536,
    ),
    (
        "Ryzen 9 7945HX",
        "AMD Ryzen 9 7945HX",
        "AMD Radeon 610M",
        32768,
    ),
];

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

    pub fn seed_defaults(&self) -> Result<()> {
        for (name, cpu, igpu, ram) in SEED_PROFILES {
            self.conn.execute(
                "INSERT OR IGNORE INTO profiles (name, cpu_model, igpu, ram_mb, os) VALUES (?1, ?2, ?3, ?4, 'Linux')",
                rusqlite::params![name, cpu, igpu, *ram as i64],
            )?;
        }
        Ok(())
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
