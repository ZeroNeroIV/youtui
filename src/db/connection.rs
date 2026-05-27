use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::info;

pub trait Filterable {
    fn matches_query(&self, query: &str) -> bool;
}

impl Filterable for HistoryEntry {
    fn matches_query(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.title.to_lowercase().contains(&q)
            || self.channel.as_ref().map(|c| c.to_lowercase().contains(&q)).unwrap_or(false)
            || self.video_id.to_lowercase().contains(&q)
    }
}

impl Filterable for SavedVideo {
    fn matches_query(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.title.to_lowercase().contains(&q)
            || self.channel.as_ref().map(|c| c.to_lowercase().contains(&q)).unwrap_or(false)
            || self.video_id.to_lowercase().contains(&q)
    }
}

pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub video_id: String,
    pub title: String,
    pub channel: Option<String>,
    pub watched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedVideo {
    pub id: i64,
    pub video_id: String,
    pub title: String,
    pub channel: Option<String>,
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub is_imported: bool,
    pub youtube_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistVideo {
    pub id: i64,
    pub playlist_id: i64,
    pub video_id: String,
    pub title: String,
    pub channel: Option<String>,
    pub position: i32,
}

impl Filterable for Playlist {
    fn matches_query(&self, query: &str) -> bool {
        self.name.to_lowercase().contains(&query.to_lowercase())
    }
}

impl Filterable for PlaylistVideo {
    fn matches_query(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.title.to_lowercase().contains(&q)
            || self.channel.as_ref().map(|c| c.to_lowercase().contains(&q)).unwrap_or(false)
            || self.video_id.to_lowercase().contains(&q)
    }
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = if let Some(data_dir) = dirs::data_dir() {
            let dir = data_dir.join("youtui-rs");
            std::fs::create_dir_all(&dir).ok();
            dir.join("youtui.db")
        } else {
            PathBuf::from("youtui.db")
        };
        let conn = Connection::open(&db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let current_version: i32 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

        let target_version = 2;

        if current_version < target_version {
            conn.execute_batch(
                "
                CREATE TABLE IF NOT EXISTS watch_history (
                    id INTEGER PRIMARY KEY,
                    video_id TEXT NOT NULL,
                    title TEXT NOT NULL,
                    channel TEXT,
                    watched_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE IF NOT EXISTS saved_videos (
                    id INTEGER PRIMARY KEY,
                    video_id TEXT NOT NULL UNIQUE,
                    title TEXT NOT NULL,
                    channel TEXT,
                    saved_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE IF NOT EXISTS playlists (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    is_imported BOOLEAN DEFAULT 0,
                    youtube_id TEXT
                );
                CREATE TABLE IF NOT EXISTS playlist_videos (
                    id INTEGER PRIMARY KEY,
                    playlist_id INTEGER NOT NULL,
                    video_id TEXT NOT NULL,
                    title TEXT NOT NULL,
                    channel TEXT,
                    position INTEGER NOT NULL,
                    FOREIGN KEY (playlist_id) REFERENCES playlists(id)
                );
                ",
            )?;

            conn.pragma_update(None, "user_version", target_version)?;
        } else if current_version == 1 {
            let mut found = false;
            let mut stmt = conn.prepare("PRAGMA table_info(playlists)")?;
            let _ = stmt.query_map([], |row| {
                found = row.get::<_, String>(1)? == "is_imported";
                Ok(())
            });
            if !found {
                let _ = conn.execute(
                    "ALTER TABLE playlists ADD COLUMN is_imported BOOLEAN DEFAULT 0",
                    [],
                );
                let _ = conn.execute("ALTER TABLE playlists ADD COLUMN youtube_id TEXT", []);
            }
        }

        Ok(())
    }

    pub fn add_to_history(
        &self,
        video_id: &str,
        title: &str,
        channel: Option<&str>,
    ) -> Result<i64> {
        info!("DEBUG DB: Adding to history: {} - {}", title, video_id);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO watch_history (video_id, title, channel) VALUES (?1, ?2, ?3)",
            (video_id, title, channel),
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_history(&self, limit: i64) -> Result<Vec<HistoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, video_id, title, channel, watched_at 
             FROM watch_history 
             ORDER BY watched_at DESC 
             LIMIT ?1",
        )?;

        let mut entries = stmt
            .query_map([limit], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    video_id: row.get(1)?,
                    title: row.get(2)?,
                    channel: row.get(3)?,
                    watched_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        entries.dedup_by(|a, b| a.video_id == b.video_id);

        Ok(entries)
    }

    pub fn save_video(&self, video_id: &str, title: &str, channel: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO saved_videos (video_id, title, channel) VALUES (?1, ?2, ?3)",
            (video_id, title, channel),
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_saved_videos(&self) -> Result<Vec<SavedVideo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, video_id, title, channel, saved_at 
             FROM saved_videos 
             ORDER BY saved_at DESC",
        )?;

        let videos = stmt
            .query_map([], |row| {
                Ok(SavedVideo {
                    id: row.get(0)?,
                    video_id: row.get(1)?,
                    title: row.get(2)?,
                    channel: row.get(3)?,
                    saved_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(videos)
    }

    pub fn unsave_video(&self, video_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM saved_videos WHERE video_id = ?1", [video_id])?;
        Ok(())
    }

    pub fn create_playlist(&self, name: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO playlists (name) VALUES (?1)", [name])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_playlists(&self) -> Result<Vec<Playlist>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, name, created_at, is_imported, youtube_id FROM playlists ORDER BY created_at DESC")?;

        let playlists = stmt
            .query_map([], |row| {
                Ok(Playlist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    is_imported: row.get(3)?,
                    youtube_id: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(playlists)
    }

    pub fn delete_playlist(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM playlist_videos WHERE playlist_id = ?1", [id])?;
        conn.execute("DELETE FROM playlists WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn add_to_playlist(
        &self,
        playlist_id: i64,
        video_id: &str,
        title: &str,
        channel: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        let position: i32 = conn.query_row(
            "SELECT COALESCE(MAX(position), 0) + 1 FROM playlist_videos WHERE playlist_id = ?1",
            [playlist_id],
            |row| row.get(0),
        )?;

        conn.execute(
            "INSERT INTO playlist_videos (playlist_id, video_id, title, channel, position) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (playlist_id, video_id, title, channel, position),
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_playlist_videos(&self, playlist_id: i64) -> Result<Vec<PlaylistVideo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, playlist_id, video_id, title, channel, position 
             FROM playlist_videos 
             WHERE playlist_id = ?1 
             ORDER BY position",
        )?;

        let videos = stmt
            .query_map([playlist_id], |row| {
                Ok(PlaylistVideo {
                    id: row.get(0)?,
                    playlist_id: row.get(1)?,
                    video_id: row.get(2)?,
                    title: row.get(3)?,
                    channel: row.get(4)?,
                    position: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(videos)
    }

    pub fn clear_history(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM watch_history", [])?;
        Ok(())
    }

    pub fn remove_from_playlist(&self, playlist_id: i64, video_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM playlist_videos WHERE id = ?1", [video_id])?;
        conn.execute(
            "UPDATE playlist_videos SET position = position - 1 WHERE playlist_id = ?1 AND position > (SELECT position FROM playlist_videos WHERE id = ?2)",
            rusqlite::params![playlist_id, video_id],
        )?;
        Ok(())
    }

    pub fn create_imported_playlist(&self, name: &str, youtube_id: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO playlists (name, is_imported, youtube_id) VALUES (?1, 1, ?2)",
            [name, youtube_id],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_playlist_by_youtube_id(&self, youtube_id: &str) -> Result<Option<Playlist>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, is_imported, youtube_id FROM playlists WHERE youtube_id = ?1"
        )?;
        let result = stmt.query_row([youtube_id], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                is_imported: row.get(3)?,
                youtube_id: row.get(4)?,
            })
        });
        match result {
            Ok(p) => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn sync_imported_playlist(
        &self,
        playlist_id: i64,
        video_id: &str,
        title: &str,
        channel: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM playlist_videos WHERE playlist_id = ?1 AND video_id = ?2)",
            rusqlite::params![playlist_id, video_id],
            |row| row.get(0),
        )?;

        if exists {
            return Ok(0);
        }

        let position: i32 = conn.query_row(
            "SELECT COALESCE(MAX(position), 0) + 1 FROM playlist_videos WHERE playlist_id = ?1",
            [playlist_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO playlist_videos (playlist_id, video_id, title, channel, position) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![playlist_id, video_id, title, channel, position],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn clear_imported_playlist(&self, playlist_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM playlist_videos WHERE playlist_id = ?1",
            [playlist_id],
        )?;
        Ok(())
    }
}
