//! Public Steam community profile (persona name and avatar), fetched without
//! an API key and cached in the `profile` table so the cards can show it while
//! the vault is locked or the machine is offline.

use crate::authenticator::USER_AGENT;
use crate::error::Result;
use crate::error_code::ErrorCode;
use crate::storage::Database;
use rusqlite::{params, OptionalExtension};
use std::io::Read;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(15);
const MAX_AVATAR_BYTES: u64 = 512 * 1024;
const MAX_PROFILE_BYTES: u64 = 4 * 1024 * 1024;
const DEFAULT_AVATAR_MIME: &str = "image/jpeg";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub steam_id: u64,
    pub persona_name: String,
    /// Image bytes and their MIME type.
    pub avatar: Option<(Vec<u8>, String)>,
    pub fetched_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ProfileError {
    #[error("{}", ErrorCode::ProfileUnavailable)]
    Network,
    #[error("{}", ErrorCode::ProfileUnavailable)]
    Invalid,
}

impl ProfileError {
    pub fn code(self) -> ErrorCode {
        ErrorCode::ProfileUnavailable
    }
}

struct Parsed {
    persona_name: String,
    avatar_url: Option<String>,
}

/// Steam answers `?xml=1` with `<profile>` for a known id and `<response>`
/// holding an `<error>` otherwise. A private profile still carries its name
/// and avatar, which is all that is read, so privacy is not an error here.
fn parse(text: &str) -> std::result::Result<Parsed, ProfileError> {
    let document = roxmltree::Document::parse(text).map_err(|_| ProfileError::Invalid)?;
    let root = document.root_element();
    if root.tag_name().name() != "profile" || child_text(root, "error").is_some() {
        return Err(ProfileError::Invalid);
    }
    let persona_name = child_text(root, "steamID")
        .filter(|name| !name.is_empty())
        .ok_or(ProfileError::Invalid)?;
    let avatar_url = child_text(root, "avatarFull").filter(|url| url.starts_with("https://"));
    Ok(Parsed {
        persona_name,
        avatar_url,
    })
}

fn child_text(node: roxmltree::Node<'_, '_>, name: &str) -> Option<String> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == name)
        .map(|child| child.text().unwrap_or_default().trim().to_string())
}

fn client() -> std::result::Result<reqwest::blocking::Client, ProfileError> {
    reqwest::blocking::Client::builder()
        .connect_timeout(TIMEOUT)
        .timeout(TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
        .map_err(|_| ProfileError::Network)
}

fn read_limited(
    response: reqwest::blocking::Response,
    limit: u64,
) -> std::result::Result<(Vec<u8>, Option<String>), ProfileError> {
    if !response.status().is_success() {
        return Err(ProfileError::Network);
    }
    let mime = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let mut bytes = Vec::new();
    response
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ProfileError::Network)?;
    if bytes.len() as u64 > limit {
        return Err(ProfileError::Invalid);
    }
    Ok((bytes, mime))
}

/// Blocking: runs on a worker thread. A missing or oversized avatar leaves
/// `avatar` empty rather than failing the whole fetch.
pub fn fetch(steam_id: u64) -> std::result::Result<Profile, ProfileError> {
    let client = client()?;
    let response = client
        .get(format!(
            "https://steamcommunity.com/profiles/{steam_id}?xml=1"
        ))
        .send()
        .map_err(|_| ProfileError::Network)?;
    let (bytes, _) = read_limited(response, MAX_PROFILE_BYTES)?;
    let text = String::from_utf8(bytes).map_err(|_| ProfileError::Invalid)?;
    let parsed = parse(&text)?;
    let avatar = parsed.avatar_url.and_then(|url| {
        let response = client.get(url).send().ok()?;
        let (bytes, mime) = read_limited(response, MAX_AVATAR_BYTES).ok()?;
        let mime = mime
            .filter(|value| value.starts_with("image/"))
            .unwrap_or_else(|| DEFAULT_AVATAR_MIME.to_string());
        Some((bytes, mime))
    });
    Ok(Profile {
        steam_id,
        persona_name: parsed.persona_name,
        avatar,
        fetched_at: crate::util::now_unix() as u64,
    })
}

/// Cloning shares the same database connection.
#[derive(Debug, Clone)]
pub struct ProfileStore {
    database: Database,
}

impl ProfileStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get(&self, steam_id: u64) -> Result<Option<Profile>> {
        self.database.read("Read profile", |connection| {
            connection
                .query_row(
                    "SELECT steam_id, persona_name, avatar, avatar_mime, fetched_at
                     FROM profile WHERE steam_id = ?1",
                    params![steam_id.to_string()],
                    profile_row,
                )
                .optional()
        })
    }

    pub fn all(&self) -> Result<Vec<Profile>> {
        self.database.read("Read profiles", |connection| {
            connection
                .prepare(
                    "SELECT steam_id, persona_name, avatar, avatar_mime, fetched_at
                     FROM profile ORDER BY steam_id",
                )?
                .query_map([], profile_row)?
                .collect()
        })
    }

    pub fn put(&self, profile: &Profile) -> Result<()> {
        self.database.transaction("Save profile", |transaction| {
            let (avatar, mime) = match &profile.avatar {
                Some((bytes, mime)) => (Some(bytes.as_slice()), Some(mime.as_str())),
                None => (None, None),
            };
            transaction.execute(
                "INSERT INTO profile (steam_id, persona_name, avatar, avatar_mime, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(steam_id) DO UPDATE SET
                     persona_name = excluded.persona_name,
                     avatar = excluded.avatar,
                     avatar_mime = excluded.avatar_mime,
                     fetched_at = excluded.fetched_at",
                params![
                    profile.steam_id.to_string(),
                    profile.persona_name,
                    avatar,
                    mime,
                    profile.fetched_at as i64
                ],
            )?;
            Ok(())
        })
    }

    pub fn remove(&self, steam_id: u64) -> Result<()> {
        self.database.transaction("Remove profile", |transaction| {
            transaction.execute(
                "DELETE FROM profile WHERE steam_id = ?1",
                params![steam_id.to_string()],
            )?;
            Ok(())
        })
    }
}

fn profile_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Profile> {
    let id: String = row.get(0)?;
    let steam_id = id.parse::<u64>().map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let avatar: Option<Vec<u8>> = row.get(2)?;
    let mime: Option<String> = row.get(3)?;
    let fetched_at: i64 = row.get(4)?;
    Ok(Profile {
        steam_id,
        persona_name: row.get(1)?,
        avatar: avatar.map(|bytes| {
            (
                bytes,
                mime.unwrap_or_else(|| DEFAULT_AVATAR_MIME.to_string()),
            )
        }),
        fetched_at: fetched_at.max(0) as u64,
    })
}
