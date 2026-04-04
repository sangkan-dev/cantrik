//! Handoff file, context bundle export/import, session replay JSON (Sprint 15).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;

use crate::config::load_merged_config;
use crate::session::{
    append_message, latest_summary, list_messages_tail_for_replay, list_recent_decisions,
    open_or_create_session, session_project_fingerprint,
};
use crate::skills::list_skill_paths;

pub const CONTEXT_BUNDLE_SCHEMA_VERSION: u32 = 1;
pub const SESSION_REPLAY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum CollabError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("session: {0}")]
    Session(#[from] crate::session::SessionError),
    #[error("unsupported context bundle schema version: {0} (expected {1})")]
    ContextBundleVersion(u32, u32),
    #[error("unsupported session replay schema version: {0} (expected {1})")]
    ReplayVersion(u32, u32),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BundleMessage {
    pub ordinal: i64,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ContextBundleV1 {
    pub schema_version: u32,
    pub project_fingerprint: String,
    pub exported_at_utc: String,
    #[serde(default)]
    pub cantrik_toml: Option<String>,
    #[serde(default)]
    pub rules_md: Option<String>,
    /// Paths relative to `.cantrik/` (e.g. `skills/foo.md`).
    #[serde(default)]
    pub skill_paths_relative: Vec<String>,
    #[serde(default)]
    pub message_tail: Vec<BundleMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SessionReplayFileV1 {
    pub schema_version: u32,
    pub session_id: String,
    pub project_fingerprint: String,
    pub exported_at_utc: String,
    #[serde(default)]
    pub messages: Vec<BundleMessage>,
}

fn skill_paths_relative(project_root: &Path) -> Vec<String> {
    let dot = project_root.join(".cantrik");
    list_skill_paths(project_root)
        .into_iter()
        .filter_map(|p| {
            p.strip_prefix(&dot).ok().map(|rel| {
                rel.to_string_lossy()
                    .replace('\\', "/")
                    .trim_start_matches('/')
                    .to_string()
            })
        })
        .collect()
}

/// Build a context bundle (no API keys; project `cantrik.toml` + rules + skill paths + message tail).
pub async fn export_context_bundle(
    pool: &SqlitePool,
    project_root: &Path,
    tail_limit: i64,
) -> Result<ContextBundleV1, CollabError> {
    let session_id = open_or_create_session(pool, project_root).await?;
    let app = load_merged_config(project_root).unwrap_or_default();
    let fp = session_project_fingerprint(project_root, &app);
    let msgs = list_messages_tail_for_replay(pool, &session_id, tail_limit).await?;
    let cantrik_path = project_root.join(".cantrik").join("cantrik.toml");
    let cantrik_toml = if cantrik_path.is_file() {
        Some(fs::read_to_string(&cantrik_path)?)
    } else {
        None
    };
    let rules_md = crate::skills::load_rules_text(project_root);
    let message_tail: Vec<BundleMessage> = msgs
        .into_iter()
        .map(|m| BundleMessage {
            ordinal: m.ordinal,
            role: m.role,
            content: m.content,
            created_at: m.created_at,
        })
        .collect();

    Ok(ContextBundleV1 {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        project_fingerprint: fp,
        exported_at_utc: Utc::now().to_rfc3339(),
        cantrik_toml,
        rules_md,
        skill_paths_relative: skill_paths_relative(project_root),
        message_tail,
    })
}

pub fn serialize_context_bundle(bundle: &ContextBundleV1) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(bundle)
}

/// Apply bundle: writes under `.cantrik/`. Optionally appends one assistant message summarizing imported tail.
pub async fn import_context_bundle(
    pool: &SqlitePool,
    project_root: &Path,
    json: &str,
    seed_session: bool,
) -> Result<(), CollabError> {
    let bundle: ContextBundleV1 = serde_json::from_str(json)?;
    if bundle.schema_version != CONTEXT_BUNDLE_SCHEMA_VERSION {
        return Err(CollabError::ContextBundleVersion(
            bundle.schema_version,
            CONTEXT_BUNDLE_SCHEMA_VERSION,
        ));
    }
    let dot = project_root.join(".cantrik");
    fs::create_dir_all(&dot)?;
    if let Some(ref text) = bundle.cantrik_toml {
        fs::write(dot.join("cantrik.toml"), text)?;
    }
    if let Some(ref text) = bundle.rules_md {
        fs::write(dot.join("rules.md"), text)?;
    }
    if seed_session && !bundle.message_tail.is_empty() {
        let sid = open_or_create_session(pool, project_root).await?;
        let summary: String = bundle
            .message_tail
            .iter()
            .map(|m| {
                format!(
                    "[ordinal {} {} @ {}]\n{}",
                    m.ordinal, m.role, m.created_at, m.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n---\n");
        let body = format!(
            "(imported context bundle; source fingerprint {})\n\n{}",
            bundle.project_fingerprint, summary
        );
        append_message(pool, &sid, "assistant", &body).await?;
    }
    Ok(())
}

/// Writes `.cantrik/handoff-YYYY-MM-DD.md` (UTC date).
pub async fn write_handoff_markdown(
    pool: &SqlitePool,
    project_root: &Path,
    next_steps: Option<&str>,
) -> Result<PathBuf, CollabError> {
    let session_id = open_or_create_session(pool, project_root).await?;
    let app = load_merged_config(project_root).unwrap_or_default();
    let fp = session_project_fingerprint(project_root, &app);
    let cwd_display = project_root.display().to_string();
    let mut decisions = list_recent_decisions(pool, &session_id, 10).await?;
    decisions.reverse();
    let summary = latest_summary(pool, &session_id).await?;

    let date = Utc::now().format("%Y-%m-%d");
    let path = project_root
        .join(".cantrik")
        .join(format!("handoff-{date}.md"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut body = String::new();
    body.push_str("# Handoff\n\n");
    body.push_str("## Project\n\n");
    body.push_str(&format!("- Working directory: `{cwd_display}`\n"));
    body.push_str(&format!("- Project fingerprint: `{fp}`\n\n"));

    body.push_str("## Recent decisions\n\n");
    if decisions.is_empty() {
        body.push_str("_None recorded in `session_decisions`._\n\n");
    } else {
        for d in &decisions {
            body.push_str(&format!(
                "- ({}) {}\n",
                d.created_at,
                d.text.replace('\n', " ")
            ));
        }
        body.push('\n');
    }

    body.push_str("## Session summary (latest)\n\n");
    match summary {
        Some(s) => {
            body.push_str(&format!(
                "_Covers messages up to ordinal {}._\n\n{}\n\n",
                s.covers_up_to_ordinal, s.text
            ));
        }
        None => {
            body.push_str("_No summary stored yet._\n\n");
        }
    }

    body.push_str("## Open questions\n\n");
    body.push_str("- [ ] _Add questions for the next implementer._\n\n");

    body.push_str("## Next steps\n\n");
    if let Some(ns) = next_steps.filter(|s| !s.is_empty()) {
        body.push_str(ns);
        if !ns.ends_with('\n') {
            body.push('\n');
        }
    } else {
        body.push_str("_Next steps TBD._\n");
    }

    fs::write(&path, body)?;
    Ok(path)
}

pub async fn export_session_replay_json(
    pool: &SqlitePool,
    project_root: &Path,
    tail_limit: i64,
) -> Result<SessionReplayFileV1, CollabError> {
    let session_id = open_or_create_session(pool, project_root).await?;
    let app = load_merged_config(project_root).unwrap_or_default();
    let fp = session_project_fingerprint(project_root, &app);
    let msgs = list_messages_tail_for_replay(pool, &session_id, tail_limit).await?;
    Ok(SessionReplayFileV1 {
        schema_version: SESSION_REPLAY_SCHEMA_VERSION,
        session_id,
        project_fingerprint: fp,
        exported_at_utc: Utc::now().to_rfc3339(),
        messages: msgs
            .into_iter()
            .map(|m| BundleMessage {
                ordinal: m.ordinal,
                role: m.role,
                content: m.content,
                created_at: m.created_at,
            })
            .collect(),
    })
}

pub fn serialize_session_replay(file: &SessionReplayFileV1) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(file)
}

pub fn parse_session_replay(json: &str) -> Result<SessionReplayFileV1, CollabError> {
    let file: SessionReplayFileV1 = serde_json::from_str(json)?;
    if file.schema_version != SESSION_REPLAY_SCHEMA_VERSION {
        return Err(CollabError::ReplayVersion(
            file.schema_version,
            SESSION_REPLAY_SCHEMA_VERSION,
        ));
    }
    Ok(file)
}

pub fn format_replay_timeline(file: &SessionReplayFileV1) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Replay schema {} session {}\nproject fingerprint {}\nexported {}\n---\n",
        file.schema_version, file.session_id, file.project_fingerprint, file.exported_at_utc
    ));
    for m in &file.messages {
        out.push_str(&format!(
            "[{}] ord {} {}:\n{}\n---\n",
            m.created_at, m.ordinal, m.role, m.content
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::session::connect_pool;

    static MEMORY_DB_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn import_bundle_writes_dot_cantrik_files() {
        let _lock = MEMORY_DB_TEST_LOCK.lock().expect("memory db test lock");
        let dir = tempfile::tempdir().expect("tmpdir");
        let root = dir.path();
        let db = dir.path().join("mem.db");
        // Isolated SQLite for this test (see `session::paths::ENV_MEMORY_DB`).
        unsafe {
            std::env::set_var(crate::session::ENV_MEMORY_DB, db.to_string_lossy().as_ref());
        }
        let pool = connect_pool().await.expect("pool");
        let bundle = ContextBundleV1 {
            schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
            project_fingerprint: "fp".into(),
            exported_at_utc: "t".into(),
            cantrik_toml: Some("[collab]\nmax_files_in_report = 5\n".into()),
            rules_md: Some("# r\n".into()),
            skill_paths_relative: vec![],
            message_tail: vec![],
        };
        let json = serde_json::to_string(&bundle).expect("json");
        import_context_bundle(&pool, root, &json, false)
            .await
            .expect("import");
        assert_eq!(
            std::fs::read_to_string(root.join(".cantrik/rules.md")).unwrap(),
            "# r\n"
        );
        assert!(
            std::fs::read_to_string(root.join(".cantrik/cantrik.toml"))
                .unwrap()
                .contains("max_files_in_report")
        );
        unsafe {
            std::env::remove_var(crate::session::ENV_MEMORY_DB);
        }
    }

    #[test]
    fn context_bundle_json_roundtrip() {
        let b = ContextBundleV1 {
            schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
            project_fingerprint: "abc".into(),
            exported_at_utc: "2026-01-01T00:00:00Z".into(),
            cantrik_toml: Some("[llm]\nmodel = \"x\"\n".into()),
            rules_md: Some("# rules\n".into()),
            skill_paths_relative: vec!["skills/a.md".into()],
            message_tail: vec![BundleMessage {
                ordinal: 1,
                role: "user".into(),
                content: "hi".into(),
                created_at: "t".into(),
            }],
        };
        let s = serialize_context_bundle(&b).expect("serialize");
        let parsed: ContextBundleV1 = serde_json::from_str(&s).expect("parse");
        assert_eq!(parsed, b);
    }

    #[test]
    fn replay_timeline_contains_ordinals() {
        let f = SessionReplayFileV1 {
            schema_version: SESSION_REPLAY_SCHEMA_VERSION,
            session_id: "s".into(),
            project_fingerprint: "p".into(),
            exported_at_utc: "t".into(),
            messages: vec![BundleMessage {
                ordinal: 2,
                role: "assistant".into(),
                content: "ok".into(),
                created_at: "c".into(),
            }],
        };
        let t = format_replay_timeline(&f);
        assert!(t.contains("ord 2"));
        assert!(t.contains("ok"));
    }
}
