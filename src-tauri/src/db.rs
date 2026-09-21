//! Local SQLite database: settings, profiles, per-game configs, the clip library, markers and
//! recording-session bookkeeping (used for crash recovery). Video files are never stored in the
//! database – only their paths and metadata.
use crate::error::Result;
use crate::quality::Quality;
use crate::settings::{CameraSettings, Settings};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct Db {
    conn: Mutex<Connection>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub quality: Quality,
    pub mic_enabled: bool,
    pub system_enabled: bool,
    pub camera_enabled: bool,
    /// Executable names that auto-activate this profile.
    pub auto_apps: Vec<String>,
    pub builtin: bool,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            quality: Quality::default(),
            mic_enabled: false,
            system_enabled: true,
            camera_enabled: false,
            auto_apps: vec![],
            builtin: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GameConfig {
    pub exe: String,
    pub name: String,
    pub profile_id: String,
    pub camera: Option<CameraSettings>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Clip {
    pub id: String,
    pub path: String,
    /// recording | replay | screenshot
    pub kind: String,
    pub title: String,
    pub game: String,
    /// Unix milliseconds.
    pub created_at: i64,
    pub duration_ms: i64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: String,
    pub encoder: String,
    pub size_bytes: i64,
    pub favorite: bool,
    pub notes: String,
    pub thumb_path: String,
    pub audio_tracks: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ClipQuery {
    /// all | recording | replay | screenshot | favorites
    pub section: String,
    pub search: String,
    /// newest | oldest | largest | smallest | longest | shortest
    pub sort: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Marker {
    pub id: i64,
    pub clip_id: String,
    pub t_ms: i64,
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    pub id: String,
    pub tmp_path: String,
    pub final_path: String,
    pub kind: String,
    pub started_at: i64,
    pub game: String,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS profiles (id TEXT PRIMARY KEY, sort INTEGER NOT NULL DEFAULT 0, json TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS game_configs (exe TEXT PRIMARY KEY COLLATE NOCASE, json TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS clips (
  id TEXT PRIMARY KEY, path TEXT NOT NULL UNIQUE, kind TEXT NOT NULL, title TEXT NOT NULL DEFAULT '',
  game TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL, duration_ms INTEGER NOT NULL DEFAULT 0,
  width INTEGER NOT NULL DEFAULT 0, height INTEGER NOT NULL DEFAULT 0, fps REAL NOT NULL DEFAULT 0,
  codec TEXT NOT NULL DEFAULT '', encoder TEXT NOT NULL DEFAULT '', size_bytes INTEGER NOT NULL DEFAULT 0,
  favorite INTEGER NOT NULL DEFAULT 0, notes TEXT NOT NULL DEFAULT '', thumb_path TEXT NOT NULL DEFAULT '',
  audio_tracks INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_clips_created ON clips(created_at);
CREATE TABLE IF NOT EXISTS markers (
  id INTEGER PRIMARY KEY AUTOINCREMENT, clip_id TEXT NOT NULL, t_ms INTEGER NOT NULL, label TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_markers_clip ON markers(clip_id);
CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY, tmp_path TEXT NOT NULL, final_path TEXT NOT NULL, kind TEXT NOT NULL,
  started_at INTEGER NOT NULL, game TEXT NOT NULL DEFAULT '', state TEXT NOT NULL DEFAULT 'recording'
);
CREATE TABLE IF NOT EXISTS session_markers (
  id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL, t_ms INTEGER NOT NULL, label TEXT NOT NULL DEFAULT ''
);
";

pub fn builtin_profiles() -> Vec<Profile> {
    let mk = |id: &str, name: &str, q: Quality, mic: bool, apps: &[&str]| Profile {
        id: id.into(),
        name: name.into(),
        quality: q,
        mic_enabled: mic,
        system_enabled: true,
        camera_enabled: false,
        auto_apps: apps.iter().map(|s| s.to_string()).collect(),
        builtin: true,
    };
    let competitive = Quality {
        preset: "custom".into(),
        height: 1440,
        fps: 120,
        bitrate_kbps: Quality::recommended_bitrate(1440, 120, "av1"),
        codec: "av1".into(),
        ..Quality::default()
    };
    let tutorial = Quality {
        preset: "custom".into(),
        height: 1440,
        fps: 60,
        ..Quality::from_preset("high")
    };
    vec![
        mk(
            "profile-default",
            "Normal Games",
            Quality::from_preset("high"),
            false,
            &[],
        ),
        mk(
            "profile-competitive",
            "Competitive Games",
            competitive,
            false,
            &[],
        ),
        mk("profile-tutorial", "Desktop Tutorial", tutorial, true, &[]),
    ]
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        Self::init(Connection::open(path)?)
    }

    pub fn open_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(SCHEMA)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.seed_profiles()?;
        Ok(db)
    }

    fn seed_profiles(&self) -> Result<()> {
        let c = self.conn.lock();
        let n: i64 = c.query_row("SELECT COUNT(*) FROM profiles", [], |r| r.get(0))?;
        if n == 0 {
            for (i, p) in builtin_profiles().iter().enumerate() {
                c.execute(
                    "INSERT INTO profiles(id, sort, json) VALUES (?1, ?2, ?3)",
                    params![p.id, i as i64, serde_json::to_string(p)?],
                )?;
            }
        }
        Ok(())
    }

    // ---- settings -------------------------------------------------------------------------------
    pub fn load_settings(&self) -> Settings {
        let raw: Option<String> = self
            .conn
            .lock()
            .query_row("SELECT value FROM kv WHERE key='settings'", [], |r| {
                r.get(0)
            })
            .optional()
            .unwrap_or(None);
        let mut s: Settings = raw
            .and_then(|r| serde_json::from_str(&r).ok())
            .unwrap_or_default();
        s.sanitize();
        s
    }

    pub fn save_settings(&self, s: &Settings) -> Result<()> {
        self.conn.lock().execute(
            "INSERT INTO kv(key,value) VALUES('settings',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![serde_json::to_string(s)?],
        )?;
        Ok(())
    }

    pub fn kv_get(&self, key: &str) -> Option<String> {
        self.conn
            .lock()
            .query_row("SELECT value FROM kv WHERE key=?1", params![key], |r| {
                r.get(0)
            })
            .optional()
            .unwrap_or(None)
    }
    pub fn kv_set(&self, key: &str, value: &str) -> Result<()> {
        self.conn.lock().execute(
            "INSERT INTO kv(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    // ---- profiles -------------------------------------------------------------------------------
    pub fn list_profiles(&self) -> Result<Vec<Profile>> {
        let c = self.conn.lock();
        let mut st = c.prepare("SELECT json FROM profiles ORDER BY sort, rowid")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        Ok(rows
            .flatten()
            .filter_map(|j| serde_json::from_str(&j).ok())
            .collect())
    }

    pub fn get_profile(&self, id: &str) -> Option<Profile> {
        let j: Option<String> = self
            .conn
            .lock()
            .query_row("SELECT json FROM profiles WHERE id=?1", params![id], |r| {
                r.get(0)
            })
            .optional()
            .ok()
            .flatten();
        j.and_then(|j| serde_json::from_str(&j).ok())
    }

    pub fn save_profile(&self, p: &Profile) -> Result<()> {
        let c = self.conn.lock();
        let sort: i64 = c.query_row("SELECT COALESCE((SELECT sort FROM profiles WHERE id=?1),(SELECT COALESCE(MAX(sort),0)+1 FROM profiles))", params![p.id], |r| r.get(0))?;
        c.execute(
            "INSERT INTO profiles(id,sort,json) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET json=excluded.json",
            params![p.id, sort, serde_json::to_string(p)?],
        )?;
        Ok(())
    }

    /// The last remaining profile cannot be deleted so there is always something to record with.
    pub fn delete_profile(&self, id: &str) -> Result<bool> {
        let c = self.conn.lock();
        let n: i64 = c.query_row("SELECT COUNT(*) FROM profiles", [], |r| r.get(0))?;
        if n <= 1 {
            return Ok(false);
        }
        Ok(c.execute("DELETE FROM profiles WHERE id=?1", params![id])? > 0)
    }

    /// Profile whose `auto_apps` list contains this executable (case-insensitive).
    pub fn profile_for_exe(&self, exe: &str) -> Option<Profile> {
        if let Some(g) = self.get_game_config(exe) {
            if let Some(p) = self.get_profile(&g.profile_id) {
                return Some(p);
            }
        }
        self.list_profiles()
            .ok()?
            .into_iter()
            .find(|p| p.auto_apps.iter().any(|a| a.eq_ignore_ascii_case(exe)))
    }

    // ---- game configs ---------------------------------------------------------------------------
    pub fn list_game_configs(&self) -> Result<Vec<GameConfig>> {
        let c = self.conn.lock();
        let mut st = c.prepare("SELECT json FROM game_configs ORDER BY exe")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        Ok(rows
            .flatten()
            .filter_map(|j| serde_json::from_str(&j).ok())
            .collect())
    }
    pub fn get_game_config(&self, exe: &str) -> Option<GameConfig> {
        let j: Option<String> = self
            .conn
            .lock()
            .query_row(
                "SELECT json FROM game_configs WHERE exe=?1",
                params![exe],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        j.and_then(|j| serde_json::from_str(&j).ok())
    }
    pub fn save_game_config(&self, g: &GameConfig) -> Result<()> {
        self.conn.lock().execute(
            "INSERT INTO game_configs(exe,json) VALUES(?1,?2) ON CONFLICT(exe) DO UPDATE SET json=excluded.json",
            params![g.exe, serde_json::to_string(g)?],
        )?;
        Ok(())
    }
    pub fn delete_game_config(&self, exe: &str) -> Result<()> {
        self.conn
            .lock()
            .execute("DELETE FROM game_configs WHERE exe=?1", params![exe])?;
        Ok(())
    }

    // ---- clips ----------------------------------------------------------------------------------
    pub fn upsert_clip(&self, c: &Clip) -> Result<()> {
        self.conn.lock().execute(
            "INSERT INTO clips(id,path,kind,title,game,created_at,duration_ms,width,height,fps,codec,encoder,size_bytes,favorite,notes,thumb_path,audio_tracks)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)
             ON CONFLICT(id) DO UPDATE SET path=excluded.path,title=excluded.title,game=excluded.game,duration_ms=excluded.duration_ms,
               width=excluded.width,height=excluded.height,fps=excluded.fps,codec=excluded.codec,encoder=excluded.encoder,
               size_bytes=excluded.size_bytes,thumb_path=excluded.thumb_path,audio_tracks=excluded.audio_tracks",
            params![c.id, c.path, c.kind, c.title, c.game, c.created_at, c.duration_ms, c.width, c.height, c.fps, c.codec, c.encoder, c.size_bytes, c.favorite as i64, c.notes, c.thumb_path, c.audio_tracks],
        )?;
        Ok(())
    }

    fn row_to_clip(r: &rusqlite::Row) -> rusqlite::Result<Clip> {
        Ok(Clip {
            id: r.get("id")?,
            path: r.get("path")?,
            kind: r.get("kind")?,
            title: r.get("title")?,
            game: r.get("game")?,
            created_at: r.get("created_at")?,
            duration_ms: r.get("duration_ms")?,
            width: r.get("width")?,
            height: r.get("height")?,
            fps: r.get("fps")?,
            codec: r.get("codec")?,
            encoder: r.get("encoder")?,
            size_bytes: r.get("size_bytes")?,
            favorite: r.get::<_, i64>("favorite")? != 0,
            notes: r.get("notes")?,
            thumb_path: r.get("thumb_path")?,
            audio_tracks: r.get("audio_tracks")?,
        })
    }

    pub fn query_clips(&self, q: &ClipQuery) -> Result<Vec<Clip>> {
        let mut sql = String::from("SELECT * FROM clips WHERE 1=1");
        let mut args: Vec<String> = vec![];
        match q.section.as_str() {
            "recording" | "replay" | "screenshot" => {
                sql.push_str(" AND kind = ?");
                args.push(q.section.clone());
            }
            "favorites" => sql.push_str(" AND favorite = 1"),
            _ => {}
        }
        let s = q.search.trim();
        if !s.is_empty() {
            sql.push_str(" AND (title LIKE ? ESCAPE '\\' OR game LIKE ? ESCAPE '\\' OR notes LIKE ? ESCAPE '\\')");
            let like = format!(
                "%{}%",
                s.replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
            );
            args.push(like.clone());
            args.push(like.clone());
            args.push(like);
        }
        sql.push_str(match q.sort.as_str() {
            "oldest" => " ORDER BY created_at ASC",
            "largest" => " ORDER BY size_bytes DESC",
            "smallest" => " ORDER BY size_bytes ASC",
            "longest" => " ORDER BY duration_ms DESC",
            "shortest" => " ORDER BY duration_ms ASC",
            _ => " ORDER BY created_at DESC",
        });
        let c = self.conn.lock();
        let mut st = c.prepare(&sql)?;
        let rows = st.query_map(rusqlite::params_from_iter(args.iter()), Self::row_to_clip)?;
        Ok(rows.flatten().collect())
    }

    pub fn get_clip(&self, id: &str) -> Option<Clip> {
        self.conn
            .lock()
            .query_row(
                "SELECT * FROM clips WHERE id=?1",
                params![id],
                Self::row_to_clip,
            )
            .optional()
            .ok()
            .flatten()
    }
    pub fn clip_by_path(&self, path: &str) -> Option<Clip> {
        self.conn
            .lock()
            .query_row(
                "SELECT * FROM clips WHERE path=?1",
                params![path],
                Self::row_to_clip,
            )
            .optional()
            .ok()
            .flatten()
    }
    pub fn all_clips(&self) -> Result<Vec<Clip>> {
        self.query_clips(&ClipQuery::default())
    }
    pub fn set_favorite(&self, id: &str, fav: bool) -> Result<()> {
        self.conn.lock().execute(
            "UPDATE clips SET favorite=?2 WHERE id=?1",
            params![id, fav as i64],
        )?;
        Ok(())
    }
    pub fn set_thumb(&self, id: &str, path: &str) -> Result<()> {
        self.conn.lock().execute(
            "UPDATE clips SET thumb_path=?2 WHERE id=?1",
            params![id, path],
        )?;
        Ok(())
    }
    pub fn set_notes(&self, id: &str, notes: &str) -> Result<()> {
        self.conn
            .lock()
            .execute("UPDATE clips SET notes=?2 WHERE id=?1", params![id, notes])?;
        Ok(())
    }
    pub fn set_clip_path(&self, id: &str, path: &str, title: &str) -> Result<()> {
        self.conn.lock().execute(
            "UPDATE clips SET path=?2, title=?3 WHERE id=?1",
            params![id, path, title],
        )?;
        Ok(())
    }
    pub fn delete_clip(&self, id: &str) -> Result<()> {
        let c = self.conn.lock();
        c.execute("DELETE FROM markers WHERE clip_id=?1", params![id])?;
        c.execute("DELETE FROM clips WHERE id=?1", params![id])?;
        Ok(())
    }

    // ---- markers --------------------------------------------------------------------------------
    pub fn add_marker(&self, clip_id: &str, t_ms: i64, label: &str) -> Result<i64> {
        let c = self.conn.lock();
        c.execute(
            "INSERT INTO markers(clip_id,t_ms,label) VALUES(?1,?2,?3)",
            params![clip_id, t_ms, label],
        )?;
        Ok(c.last_insert_rowid())
    }
    pub fn list_markers(&self, clip_id: &str) -> Result<Vec<Marker>> {
        let c = self.conn.lock();
        let mut st =
            c.prepare("SELECT id,clip_id,t_ms,label FROM markers WHERE clip_id=?1 ORDER BY t_ms")?;
        let rows = st.query_map(params![clip_id], |r| {
            Ok(Marker {
                id: r.get(0)?,
                clip_id: r.get(1)?,
                t_ms: r.get(2)?,
                label: r.get(3)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }
    pub fn delete_marker(&self, id: i64) -> Result<()> {
        self.conn
            .lock()
            .execute("DELETE FROM markers WHERE id=?1", params![id])?;
        Ok(())
    }
    pub fn add_session_marker(&self, session_id: &str, t_ms: i64, label: &str) -> Result<()> {
        self.conn.lock().execute(
            "INSERT INTO session_markers(session_id,t_ms,label) VALUES(?1,?2,?3)",
            params![session_id, t_ms, label],
        )?;
        Ok(())
    }
    /// Moves markers captured during a live session onto the finished clip.
    pub fn promote_session_markers(&self, session_id: &str, clip_id: &str) -> Result<()> {
        let c = self.conn.lock();
        c.execute(
            "INSERT INTO markers(clip_id,t_ms,label) SELECT ?2,t_ms,label FROM session_markers WHERE session_id=?1",
            params![session_id, clip_id],
        )?;
        c.execute(
            "DELETE FROM session_markers WHERE session_id=?1",
            params![session_id],
        )?;
        Ok(())
    }

    // ---- sessions (crash recovery) ---------------------------------------------------------------
    pub fn begin_session(&self, s: &SessionRow) -> Result<()> {
        self.conn.lock().execute(
            "INSERT OR REPLACE INTO sessions(id,tmp_path,final_path,kind,started_at,game,state) VALUES(?1,?2,?3,?4,?5,?6,'recording')",
            params![s.id, s.tmp_path, s.final_path, s.kind, s.started_at, s.game],
        )?;
        Ok(())
    }
    pub fn end_session(&self, id: &str, state: &str) -> Result<()> {
        self.conn.lock().execute(
            "UPDATE sessions SET state=?2 WHERE id=?1",
            params![id, state],
        )?;
        Ok(())
    }
    /// Sessions that were still 'recording' when the app last exited (crash / power loss).
    pub fn interrupted_sessions(&self) -> Result<Vec<SessionRow>> {
        let c = self.conn.lock();
        let mut st = c.prepare("SELECT id,tmp_path,final_path,kind,started_at,game FROM sessions WHERE state='recording'")?;
        let rows = st.query_map([], |r| {
            Ok(SessionRow {
                id: r.get(0)?,
                tmp_path: r.get(1)?,
                final_path: r.get(2)?,
                kind: r.get(3)?,
                started_at: r.get(4)?,
                game: r.get(5)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }
    pub fn session_history(&self, limit: u32) -> Result<Vec<SessionRow>> {
        let c = self.conn.lock();
        let mut st = c.prepare("SELECT id,tmp_path,final_path,kind,started_at,game FROM sessions ORDER BY started_at DESC LIMIT ?1")?;
        let rows = st.query_map(params![limit], |r| {
            Ok(SessionRow {
                id: r.get(0)?,
                tmp_path: r.get(1)?,
                final_path: r.get(2)?,
                kind: r.get(3)?,
                started_at: r.get(4)?,
                game: r.get(5)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(id: &str, kind: &str, title: &str, t: i64, size: i64, dur: i64) -> Clip {
        Clip {
            id: id.into(),
            path: format!("C:/clips/{id}.mp4"),
            kind: kind.into(),
            title: title.into(),
            game: "Game".into(),
            created_at: t,
            size_bytes: size,
            duration_ms: dur,
            ..Clip::default()
        }
    }

    #[test]
    fn seeds_three_builtin_profiles() {
        let db = Db::open_memory().unwrap();
        let p = db.list_profiles().unwrap();
        assert_eq!(p.len(), 3);
        assert_eq!(p[1].quality.codec, "av1");
    }

    #[test]
    fn settings_persist() {
        let db = Db::open_memory().unwrap();
        let mut s = db.load_settings();
        s.general.language = "ar".into();
        db.save_settings(&s).unwrap();
        assert_eq!(db.load_settings().general.language, "ar");
    }

    #[test]
    fn last_profile_cannot_be_deleted() {
        let db = Db::open_memory().unwrap();
        assert!(db.delete_profile("profile-tutorial").unwrap());
        assert!(db.delete_profile("profile-competitive").unwrap());
        assert!(!db.delete_profile("profile-default").unwrap());
    }

    #[test]
    fn profile_lookup_by_exe_prefers_game_config() {
        let db = Db::open_memory().unwrap();
        let mut p = db.get_profile("profile-tutorial").unwrap();
        p.auto_apps = vec!["obsidian.exe".into()];
        db.save_profile(&p).unwrap();
        assert_eq!(
            db.profile_for_exe("Obsidian.EXE").unwrap().id,
            "profile-tutorial"
        );
        db.save_game_config(&GameConfig {
            exe: "game.exe".into(),
            name: "Game".into(),
            profile_id: "profile-competitive".into(),
            camera: None,
        })
        .unwrap();
        assert_eq!(
            db.profile_for_exe("GAME.exe").unwrap().id,
            "profile-competitive"
        );
        assert!(db.profile_for_exe("nothing.exe").is_none());
    }

    #[test]
    fn library_sections_search_and_sort() {
        let db = Db::open_memory().unwrap();
        db.upsert_clip(&clip("a", "recording", "Boss fight", 1, 500, 10_000))
            .unwrap();
        db.upsert_clip(&clip("b", "replay", "Clutch", 2, 100, 30_000))
            .unwrap();
        db.upsert_clip(&clip("c", "screenshot", "Menu", 3, 900, 0))
            .unwrap();
        db.set_favorite("b", true).unwrap();

        let q = |section: &str, search: &str, sort: &str| ClipQuery {
            section: section.into(),
            search: search.into(),
            sort: sort.into(),
        };
        assert_eq!(db.query_clips(&q("all", "", "newest")).unwrap()[0].id, "c");
        assert_eq!(db.query_clips(&q("all", "", "oldest")).unwrap()[0].id, "a");
        assert_eq!(db.query_clips(&q("all", "", "largest")).unwrap()[0].id, "c");
        assert_eq!(db.query_clips(&q("all", "", "longest")).unwrap()[0].id, "b");
        assert_eq!(db.query_clips(&q("replay", "", "newest")).unwrap().len(), 1);
        assert_eq!(
            db.query_clips(&q("favorites", "", "newest")).unwrap()[0].id,
            "b"
        );
        assert_eq!(
            db.query_clips(&q("all", "boss", "newest")).unwrap().len(),
            1
        );
        assert_eq!(
            db.query_clips(&q("all", "100%", "newest")).unwrap().len(),
            0
        );
    }

    #[test]
    fn markers_follow_the_clip() {
        let db = Db::open_memory().unwrap();
        db.upsert_clip(&clip("a", "recording", "x", 1, 1, 1))
            .unwrap();
        db.add_session_marker("s1", 5_000, "ace").unwrap();
        db.add_session_marker("s1", 1_000, "start").unwrap();
        db.promote_session_markers("s1", "a").unwrap();
        let m = db.list_markers("a").unwrap();
        assert_eq!(
            m.iter().map(|m| m.t_ms).collect::<Vec<_>>(),
            vec![1_000, 5_000]
        );
        db.delete_clip("a").unwrap();
        assert!(db.list_markers("a").unwrap().is_empty());
    }

    #[test]
    fn interrupted_sessions_are_reported_until_closed() {
        let db = Db::open_memory().unwrap();
        let s = SessionRow {
            id: "s".into(),
            tmp_path: "t.mkv".into(),
            final_path: "f.mp4".into(),
            kind: "recording".into(),
            started_at: 1,
            game: "".into(),
        };
        db.begin_session(&s).unwrap();
        assert_eq!(db.interrupted_sessions().unwrap().len(), 1);
        db.end_session("s", "done").unwrap();
        assert!(db.interrupted_sessions().unwrap().is_empty());
    }
}
