use serde::{Deserialize, Serialize};
use rusqlite::{Connection, params, OptionalExtension};
use tauri::{AppHandle, Manager};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::multipart::{Form, Part};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct Question {
    pub id: i64,
    pub part: i32,
    pub sub_part: i32,
    pub audio_path: String,
    pub image_path: String,
    pub text: String,
    pub pack_id: String,
    pub pack_order: i32,
    pub response_time: i32,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attempt {
    pub id: i64,
    pub student_name: String,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: i64,
    pub attempt_id: i64,
    pub question_id: i64,
    pub audio_path: String,
    pub duration: i32,
    pub recorded_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Recording {
    pub id: i64,
    pub attempt_id: i64,
    pub student_name: String,
    pub attempt_started_at: String,
    pub question_id: i64,
    pub part: i32,
    pub sub_part: i32,
    pub question_text: String,
    pub recorded_at: String,
    pub duration: i32,
}

const DEFAULT_TELEGRAM_BOT_TOKEN: &str = "8505694385:AAELharS3HfJoSc4CmAxinNFRR-LteH5dIA";
const TELEGRAM_API_BASE: &str = "https://api.telegram.org";
const TELEGRAM_CHAT_ID_SETTING_KEY: &str = "telegram_chat_id";
const TELEGRAM_CHAT_IDS_SETTING_KEY: &str = "telegram_chat_ids";

#[derive(Deserialize, Debug)]
struct TelegramApiResponse {
    ok: bool,
    description: Option<String>,
}

// Initialize database connection
fn init_db(app_handle: &AppHandle) -> Result<Connection, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    let db_path = app_dir.join("exam.db");
    
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    // Create tables if they don't exist
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS attempts (
            id INTEGER PRIMARY KEY,
            student_name TEXT NOT NULL,
            started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            finished_at DATETIME
        );
        
        CREATE TABLE IF NOT EXISTS questions (
            id INTEGER PRIMARY KEY,
            part INTEGER NOT NULL,
            sub_part INTEGER NOT NULL DEFAULT 0,
            audio_path TEXT NOT NULL,
            image_path TEXT NOT NULL DEFAULT '',
            text TEXT NOT NULL DEFAULT '',
            pack_id TEXT NOT NULL DEFAULT '',
            pack_order INTEGER NOT NULL DEFAULT 0,
            response_time INTEGER NOT NULL,
            active BOOLEAN DEFAULT 1
        );
        
        CREATE TABLE IF NOT EXISTS responses (
            id INTEGER PRIMARY KEY,
            attempt_id INTEGER NOT NULL,
            question_id INTEGER NOT NULL,
            audio_path TEXT NOT NULL,
            duration INTEGER NOT NULL,
            recorded_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (attempt_id) REFERENCES attempts (id),
            FOREIGN KEY (question_id) REFERENCES questions (id)
        );
        
        CREATE TABLE IF NOT EXISTS ratings (
            response_id INTEGER PRIMARY KEY,
            fluency INTEGER CHECK (fluency BETWEEN 1 AND 9),
            lexical INTEGER CHECK (lexical BETWEEN 1 AND 9),
            grammar INTEGER CHECK (grammar BETWEEN 1 AND 9),
            pronunciation INTEGER CHECK (pronunciation BETWEEN 1 AND 9),
            comment TEXT,
            FOREIGN KEY (response_id) REFERENCES responses (id)
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        );"
    ).map_err(|e| e.to_string())?;
    
    ensure_column(&conn, "questions", "sub_part", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(&conn, "questions", "image_path", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(&conn, "questions", "text", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(&conn, "questions", "pack_id", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(&conn, "questions", "pack_order", "INTEGER NOT NULL DEFAULT 0")?;
    conn.execute(
        "UPDATE questions SET sub_part = 1 WHERE part = 1 AND sub_part = 0",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(conn)
}

fn ensure_column(conn: &Connection, table: &str, column: &str, column_def: &str) -> Result<(), String> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let name: String = row.get(1).map_err(|e| e.to_string())?;
        if name == column {
            return Ok(());
        }
    }

    conn.execute(
        &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, column_def),
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

fn unique_filename(dir: &Path, filename: &str) -> Result<String, String> {
    let base_name = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?
        .to_string();

    if !dir.join(&base_name).exists() {
        return Ok(base_name);
    }

    let path = Path::new(&base_name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    for _ in 0..50 {
        let suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        let candidate = if ext.is_empty() {
            format!("{}_{}", stem, suffix)
        } else {
            format!("{}_{}.{}", stem, suffix, ext)
        };
        if !dir.join(&candidate).exists() {
            return Ok(candidate);
        }
    }

    Err("Failed to generate unique filename".to_string())
}

fn telegram_bot_token() -> String {
    std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_else(|_| DEFAULT_TELEGRAM_BOT_TOKEN.to_string())
}

fn normalize_telegram_chat_ids(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        let candidate = trimmed.to_string();
        if seen.insert(candidate.clone()) {
            normalized.push(candidate);
        }
    }

    normalized
}

fn parse_telegram_chat_ids(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if trimmed.starts_with('[') {
        if let Ok(values) = serde_json::from_str::<Vec<String>>(trimmed) {
            return normalize_telegram_chat_ids(values);
        }
    }

    let values = trimmed
        .split(|c| c == ',' || c == ';' || c == '\n')
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    normalize_telegram_chat_ids(values)
}

fn env_telegram_chat_ids() -> Vec<String> {
    if let Ok(value) = std::env::var("TELEGRAM_CHAT_IDS") {
        let ids = parse_telegram_chat_ids(&value);
        if !ids.is_empty() {
            return ids;
        }
    }

    if let Ok(value) = std::env::var("TELEGRAM_CHAT_ID") {
        return parse_telegram_chat_ids(&value);
    }

    Vec::new()
}

fn stored_telegram_chat_ids(conn: &Connection) -> Result<Option<Vec<String>>, String> {
    let saved_chat_ids_raw: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?",
            params![TELEGRAM_CHAT_IDS_SETTING_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(value) = saved_chat_ids_raw {
        return Ok(Some(parse_telegram_chat_ids(&value)));
    }

    let legacy_chat_id_raw: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?",
            params![TELEGRAM_CHAT_ID_SETTING_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(legacy_chat_id_raw.map(|value| parse_telegram_chat_ids(&value)))
}

fn persist_telegram_chat_ids(conn: &Connection, chat_ids: &[String]) -> Result<(), String> {
    let serialized_ids =
        serde_json::to_string(chat_ids).map_err(|e| format!("Failed to serialize chat IDs: {}", e))?;
    let primary_chat_id = chat_ids
        .first()
        .map(|value| value.as_str())
        .unwrap_or_default();

    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![TELEGRAM_CHAT_IDS_SETTING_KEY, serialized_ids],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![TELEGRAM_CHAT_ID_SETTING_KEY, primary_chat_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn load_telegram_chat_ids(conn: &Connection) -> Result<Vec<String>, String> {
    if let Some(chat_ids) = stored_telegram_chat_ids(conn)? {
        if chat_ids.is_empty() {
            return Err("Telegram chat IDs are not configured. Add at least one in Admin Dashboard.".to_string());
        }
        return Ok(chat_ids);
    }

    let env_chat_ids = env_telegram_chat_ids();
    if !env_chat_ids.is_empty() {
        return Ok(env_chat_ids);
    }

    Err("Telegram chat IDs are not configured. Add at least one in Admin Dashboard.".to_string())
}

fn truncate_for_telegram(value: &str, max_chars: usize) -> String {
    let mut iter = value.chars();
    let mut truncated: String = iter.by_ref().take(max_chars).collect();
    if iter.next().is_some() {
        if max_chars > 3 {
            truncated.truncate(max_chars.saturating_sub(3));
            truncated.push_str("...");
        }
    }
    truncated
}

fn format_part_label(part: i32, sub_part: i32) -> String {
    if part == 1 && (sub_part == 1 || sub_part == 2) {
        return format!("Part 1.{}", sub_part);
    }
    format!("Part {}", part)
}

fn resolve_image_path(app_dir: &Path, image_path: &str) -> Option<PathBuf> {
    let trimmed = image_path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Some(path.to_path_buf());
    }

    Some(app_dir.join("images").join(trimmed))
}

fn guess_mime(path: &Path, is_image: bool) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "webm" => "audio/webm",
        _ if is_image => "application/octet-stream",
        _ => "application/octet-stream",
    }
}

fn convert_audio_to_mp3(source_path: &Path) -> Result<PathBuf, String> {
    if !source_path.exists() {
        return Err(format!(
            "Cannot convert to MP3 because source file does not exist: {:?}",
            source_path
        ));
    }

    let ffmpeg_available = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !ffmpeg_available {
        return Err(
            "ffmpeg is not installed or not available in PATH. Install ffmpeg to send MP3 files."
                .to_string(),
        );
    }

    let stem = source_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("response");
    let suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let output_path = std::env::temp_dir().join(format!("{}_{}_telegram.mp3", stem, suffix));

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(source_path)
        .arg("-vn")
        .arg("-codec:a")
        .arg("libmp3lame")
        .arg("-q:a")
        .arg("2")
        .arg(&output_path)
        .status()
        .map_err(|e| format!("Failed to run ffmpeg for MP3 conversion: {}", e))?;

    if !status.success() {
        return Err("ffmpeg failed to convert recording to MP3".to_string());
    }

    if !output_path.exists() {
        return Err("MP3 conversion finished but output file was not created".to_string());
    }

    Ok(output_path)
}

async fn parse_telegram_response(
    response: reqwest::Response,
    endpoint: &str,
) -> Result<(), String> {
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read Telegram {} response: {}", endpoint, e))?;

    if !status.is_success() {
        return Err(format!(
            "Telegram {} failed with HTTP {}: {}",
            endpoint, status, body
        ));
    }

    let parsed: TelegramApiResponse = serde_json::from_str(&body).map_err(|e| {
        format!(
            "Failed to parse Telegram {} response JSON: {} (body: {})",
            endpoint, e, body
        )
    })?;

    if !parsed.ok {
        return Err(format!(
            "Telegram {} returned error: {}",
            endpoint,
            parsed
                .description
                .unwrap_or_else(|| "Unknown Telegram error".to_string())
        ));
    }

    Ok(())
}

async fn send_telegram_message(
    client: &reqwest::Client,
    bot_token: &str,
    chat_id: &str,
    text: &str,
) -> Result<(), String> {
    let url = format!("{}/bot{}/sendMessage", TELEGRAM_API_BASE, bot_token);
    let response = client
        .post(url)
        .form(&[("chat_id", chat_id), ("text", text)])
        .send()
        .await
        .map_err(|e| format!("Failed to call Telegram sendMessage: {}", e))?;

    parse_telegram_response(response, "sendMessage").await
}

async fn send_telegram_photo(
    client: &reqwest::Client,
    bot_token: &str,
    chat_id: &str,
    image_path: &Path,
    caption: &str,
) -> Result<(), String> {
    let image_data =
        fs::read(image_path).map_err(|e| format!("Failed to read image file for Telegram: {}", e))?;
    let image_name = image_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("prompt_image.bin")
        .to_string();

    let image_part = Part::bytes(image_data)
        .file_name(image_name)
        .mime_str(guess_mime(image_path, true))
        .map_err(|e| format!("Failed to set Telegram image MIME type: {}", e))?;

    let mut form = Form::new()
        .text("chat_id", chat_id.to_string())
        .part("photo", image_part);

    if !caption.trim().is_empty() {
        form = form.text("caption", caption.to_string());
    }

    let url = format!("{}/bot{}/sendPhoto", TELEGRAM_API_BASE, bot_token);
    let response = client
        .post(url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Failed to call Telegram sendPhoto: {}", e))?;

    parse_telegram_response(response, "sendPhoto").await
}

async fn send_telegram_document(
    client: &reqwest::Client,
    bot_token: &str,
    chat_id: &str,
    file_path: &Path,
    caption: &str,
) -> Result<(), String> {
    let file_data =
        fs::read(file_path).map_err(|e| format!("Failed to read recording for Telegram: {}", e))?;
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("response.webm")
        .to_string();

    let document_part = Part::bytes(file_data)
        .file_name(file_name)
        .mime_str(guess_mime(file_path, false))
        .map_err(|e| format!("Failed to set Telegram document MIME type: {}", e))?;

    let mut form = Form::new()
        .text("chat_id", chat_id.to_string())
        .part("document", document_part);

    if !caption.trim().is_empty() {
        form = form.text("caption", caption.to_string());
    }

    let url = format!("{}/bot{}/sendDocument", TELEGRAM_API_BASE, bot_token);
    let response = client
        .post(url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Failed to call Telegram sendDocument: {}", e))?;

    parse_telegram_response(response, "sendDocument").await
}

async fn send_response_to_telegram(
    app_dir: &Path,
    chat_id: &str,
    student_name: &str,
    question_id: i64,
    part: i32,
    sub_part: i32,
    question_text: &str,
    image_path: &str,
    duration: i32,
    audio_file_path: &Path,
) -> Result<(), String> {
    let bot_token = telegram_bot_token();
    let client = reqwest::Client::new();
    let part_label = format_part_label(part, sub_part);
    let question = if question_text.trim().is_empty() {
        "(No question text provided)"
    } else {
        question_text.trim()
    };

    let message = format!(
        "New CEFR speaking response\nStudent: {}\nSection: {}\nQuestion ID: {}\nQuestion: {}\nDuration: {}s",
        student_name, part_label, question_id, question, duration
    );

    send_telegram_message(
        &client,
        &bot_token,
        chat_id,
        &truncate_for_telegram(&message, 4096),
    )
    .await?;

    if let Some(resolved_image_path) = resolve_image_path(app_dir, image_path) {
        if resolved_image_path.exists() {
            let image_caption = format!(
                "Prompt image for {} (Question {})",
                part_label, question_id
            );
            send_telegram_photo(
                &client,
                &bot_token,
                chat_id,
                &resolved_image_path,
                &truncate_for_telegram(&image_caption, 1024),
            )
            .await?;
        } else {
            println!(
                "Warning: image file configured for question {} but not found at {:?}",
                question_id, resolved_image_path
            );
        }
    }

    let mp3_caption = format!(
        "Answer recording (MP3) | {} | Question {} | Student {}",
        part_label, question_id, student_name
    );
    let fallback_caption = format!(
        "Answer recording (original format) | {} | Question {} | Student {}",
        part_label, question_id, student_name
    );

    match convert_audio_to_mp3(audio_file_path) {
        Ok(mp3_file_path) => {
            let send_result = send_telegram_document(
                &client,
                &bot_token,
                chat_id,
                &mp3_file_path,
                &truncate_for_telegram(&mp3_caption, 1024),
            )
            .await;

            if let Err(error) = fs::remove_file(&mp3_file_path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    println!(
                        "Warning: failed to delete temporary MP3 file {:?}: {}",
                        mp3_file_path, error
                    );
                }
            }

            send_result?;
        }
        Err(error) => {
            println!(
                "Warning: MP3 conversion failed. Sending original recording instead. Reason: {}",
                error
            );
            send_telegram_document(
                &client,
                &bot_token,
                chat_id,
                audio_file_path,
                &truncate_for_telegram(&fallback_caption, 1024),
            )
            .await?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_telegram_chat_id(
    app_handle: AppHandle,
) -> Result<String, String> {
    let conn = init_db(&app_handle)?;
    if let Some(chat_ids) = stored_telegram_chat_ids(&conn)? {
        return Ok(chat_ids.into_iter().next().unwrap_or_default());
    }
    Ok(env_telegram_chat_ids().into_iter().next().unwrap_or_default())
}

#[tauri::command]
pub async fn set_telegram_chat_id(
    app_handle: AppHandle,
    chat_id: String,
) -> Result<(), String> {
    let trimmed = chat_id.trim();
    if trimmed.is_empty() {
        return Err("Telegram chat ID cannot be empty".to_string());
    }

    let conn = init_db(&app_handle)?;
    persist_telegram_chat_ids(&conn, &[trimmed.to_string()])?;

    Ok(())
}

#[tauri::command]
pub async fn get_telegram_chat_ids(
    app_handle: AppHandle,
) -> Result<Vec<String>, String> {
    let conn = init_db(&app_handle)?;
    if let Some(chat_ids) = stored_telegram_chat_ids(&conn)? {
        return Ok(chat_ids);
    }
    Ok(env_telegram_chat_ids())
}

#[tauri::command]
pub async fn set_telegram_chat_ids(
    app_handle: AppHandle,
    chat_ids: Vec<String>,
) -> Result<Vec<String>, String> {
    let normalized_chat_ids = normalize_telegram_chat_ids(chat_ids);
    let conn = init_db(&app_handle)?;
    persist_telegram_chat_ids(&conn, &normalized_chat_ids)?;

    Ok(normalized_chat_ids)
}

#[tauri::command]
pub async fn create_attempt(
    app_handle: AppHandle,
    student_name: String,
) -> Result<i64, String> {
    let conn = init_db(&app_handle)?;
    
    conn.execute(
        "INSERT INTO attempts (student_name) VALUES (?)",
        params![student_name],
    ).map_err(|e| e.to_string())?;
    
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub async fn get_random_questions(
    app_handle: AppHandle,
    part: i32,
    count: i32,
    exclude_ids: Vec<i64>,
    sub_part: Option<i32>,
) -> Result<Vec<Question>, String> {
    let conn = init_db(&app_handle)?;

    let map_question = |row: &rusqlite::Row| {
        Ok(Question {
            id: row.get(0)?,
            part: row.get(1)?,
            sub_part: row.get(2)?,
            audio_path: row.get(3)?,
            image_path: row.get(4)?,
            text: row.get(5)?,
            pack_id: row.get(6)?,
            pack_order: row.get(7)?,
            response_time: row.get(8)?,
            active: row.get(9)?,
        })
    };

    if part == 1 {
        if let Some(sub_part_value) = sub_part {
            if sub_part_value == 1 || sub_part_value == 2 {
                let pack_id: Option<String> = conn
                    .query_row(
                        "SELECT pack_id FROM questions
                         WHERE part = ? AND sub_part = ? AND active = 1 AND pack_id <> ''
                         GROUP BY pack_id
                         ORDER BY RANDOM()
                         LIMIT 1",
                        params![part, sub_part_value],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|e| e.to_string())?;

                if let Some(pack_id) = pack_id {
                    let mut stmt = conn.prepare(
                        "SELECT id, part, sub_part, audio_path, image_path, text, pack_id, pack_order, response_time, active
                         FROM questions
                         WHERE part = ? AND sub_part = ? AND active = 1 AND pack_id = ?
                         ORDER BY pack_order ASC, id ASC",
                    ).map_err(|e| e.to_string())?;

                    let questions = stmt
                        .query_map(params![part, sub_part_value, pack_id], map_question)
                        .map_err(|e| e.to_string())?
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| e.to_string())?;

                    if !questions.is_empty() {
                        return Ok(questions);
                    }
                }
            }
        }
    }
    
    // Build the query
    let exclude_clause = if exclude_ids.is_empty() {
        "".to_string()
    } else {
        let ids_str = exclude_ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!("AND id NOT IN ({})", ids_str)
    };
    
    let sub_part_clause = match sub_part {
        Some(_) => "AND sub_part = ?".to_string(),
        None => "".to_string(),
    };

    let sql = format!(
        "SELECT id, part, sub_part, audio_path, image_path, text, pack_id, pack_order, response_time, active 
         FROM questions 
         WHERE part = ? AND active = 1 {} {}
         ORDER BY RANDOM() 
         LIMIT ?",
        sub_part_clause, exclude_clause
    );
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let question_iter = match sub_part {
        Some(value) => stmt.query_map(params![part, value, count], map_question),
        None => stmt.query_map(params![part, count], map_question),
    };

    let questions = question_iter
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    
    Ok(questions)
}

#[tauri::command]
pub async fn save_response(
    app_handle: AppHandle,
    attempt_id: i64,
    question_id: i64,
    audio_data: Vec<u8>,
    duration: i32,
) -> Result<String, String> {
    // Get student name for folder structure
    let conn = init_db(&app_handle)?;
    let student_name: String = conn.query_row(
        "SELECT student_name FROM attempts WHERE id = ?",
        params![attempt_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    let (question_part, question_sub_part, question_text, question_image_path): (i32, i32, String, String) =
        conn.query_row(
            "SELECT part, sub_part, text, image_path FROM questions WHERE id = ?",
            params![question_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| e.to_string())?;
    
    // Create directory structure: responses/{date}/{student_name}/
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let safe_name = student_name.replace(" ", "_")
        .replace("/", "_")
        .replace("\\", "_")
        .replace(":", "_");
    
    let dir = app_dir.join("responses")
        .join(&date)
        .join(&safe_name);
    
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    
    // Save audio file
    let filename = format!("q{}.webm", question_id);
    let filepath = dir.join(&filename);
    let filepath_str = filepath.to_string_lossy().to_string();
    
    fs::write(&filepath, audio_data).map_err(|e| e.to_string())?;
    
    // Save to database
    conn.execute(
        "INSERT INTO responses (attempt_id, question_id, audio_path, duration) 
         VALUES (?, ?, ?, ?)",
        params![attempt_id, question_id, filepath_str, duration],
    ).map_err(|e| e.to_string())?;

    let target_chat_ids = load_telegram_chat_ids(&conn)?;
    let mut send_errors = Vec::new();

    for target_chat_id in target_chat_ids {
        if let Err(error) = send_response_to_telegram(
            &app_dir,
            &target_chat_id,
            &student_name,
            question_id,
            question_part,
            question_sub_part,
            &question_text,
            &question_image_path,
            duration,
            &filepath,
        )
        .await
        {
            send_errors.push(format!("{} ({})", target_chat_id, error));
        }
    }

    if !send_errors.is_empty() {
        return Err(format!(
            "Response saved locally but failed to send to Telegram for {} chat(s): {}",
            send_errors.len(),
            send_errors.join(" | ")
        ));
    }
    
    Ok(filepath_str)
}

#[tauri::command]
pub async fn finish_attempt(
    app_handle: AppHandle,
    attempt_id: i64,
) -> Result<(), String> {
    let conn = init_db(&app_handle)?;
    
    conn.execute(
        "UPDATE attempts SET finished_at = CURRENT_TIMESTAMP WHERE id = ?",
        params![attempt_id],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_attempts(
    app_handle: AppHandle,
) -> Result<Vec<Attempt>, String> {
    let conn = init_db(&app_handle)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, student_name, started_at, finished_at FROM attempts ORDER BY started_at DESC"
    ).map_err(|e| e.to_string())?;
    
    let attempts = stmt
        .query_map([], |row| {
            Ok(Attempt {
                id: row.get(0)?,
                student_name: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    
    Ok(attempts)
}

#[tauri::command]
pub async fn save_audio_file(
    app_handle: AppHandle,
    filename: String,
    audio_data: Vec<u8>,
) -> Result<String, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let audio_dir = app_dir.join("audio");
    fs::create_dir_all(&audio_dir).map_err(|e| e.to_string())?;

    let safe_filename = unique_filename(&audio_dir, &filename)?;
    let audio_file_path = audio_dir.join(&safe_filename);
    println!("Saving audio file to: {:?}, size: {} bytes", audio_file_path, audio_data.len());
    
    fs::write(&audio_file_path, &audio_data).map_err(|e| {
        let err_msg = format!("Failed to write audio file: {}", e);
        println!("Error: {}", err_msg);
        err_msg
    })?;
    
    println!("Audio file saved successfully: {}", safe_filename);
    Ok(safe_filename)
}

#[tauri::command]
pub async fn get_audio_file(
    app_handle: AppHandle,
    filename: String,
) -> Result<Vec<u8>, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let mut candidates = Vec::new();
    candidates.push(app_dir.join("audio").join(&filename));

    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        candidates.push(resource_dir.join(&filename));
        candidates.push(resource_dir.join("audio").join(&filename));
        candidates.push(resource_dir.join("instruction_sounds").join(&filename));
        candidates.push(resource_dir.join("_up_").join("instruction_sounds").join(&filename));
        candidates.push(resource_dir.join("_up_").join("src").join("instruction_sounds").join(&filename));
    }

    let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src")
        .join("instruction_sounds")
        .join(&filename);
    candidates.push(dev_path);

    for path in candidates {
        if !path.exists() {
            continue;
        }
        println!("Reading audio file from: {:?}", path);
        let data = fs::read(&path)
            .map_err(|e| {
                let err_msg = format!("Failed to read audio file {}: {}", filename, e);
                println!("Error: {}", err_msg);
                err_msg
            })?;
        println!("Audio file read successfully: {} bytes", data.len());
        return Ok(data);
    }

    let err_msg = format!("Failed to read audio file {}: file not found", filename);
    println!("Error: {}", err_msg);
    Err(err_msg)
}

#[tauri::command]
pub async fn add_question(
    app_handle: AppHandle,
    part: i32,
    sub_part: Option<i32>,
    response_time: i32,
    audio_path: Option<String>,
    image_path: Option<String>,
    text: Option<String>,
    pack_id: Option<String>,
    pack_order: Option<i32>,
) -> Result<i64, String> {
    let conn = init_db(&app_handle)?;
    
    // Use empty string if no audio path provided
    let audio = audio_path.unwrap_or_default();
    let image = image_path.unwrap_or_default();
    let question_text = if part == 3 { String::new() } else { text.unwrap_or_default() };
    let sub_part_value = sub_part.unwrap_or(0);
    let pack_id_value = pack_id.unwrap_or_default();
    let pack_order_value = pack_order.unwrap_or(0);

    if part == 1 && sub_part_value == 2 && image.is_empty() {
        return Err("Image is required for Part 1.2 questions".to_string());
    }

    if part == 3 && image.is_empty() {
        return Err("Image is required for Part 3 questions".to_string());
    }

    if part == 1 && (sub_part_value == 1 || sub_part_value == 2) {
        if pack_id_value.trim().is_empty() {
            return Err("Test pack ID is required for Part 1.1 and Part 1.2 questions".to_string());
        }
        if pack_order_value <= 0 {
            return Err("Question order is required for Part 1.1 and Part 1.2 questions".to_string());
        }
    }
    
    conn.execute(
        "INSERT INTO questions (part, sub_part, audio_path, image_path, text, pack_id, pack_order, response_time, active) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)",
        params![part, sub_part_value, audio, image, question_text, pack_id_value, pack_order_value, response_time],
    ).map_err(|e| e.to_string())?;
    
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub async fn get_questions(
    app_handle: AppHandle,
) -> Result<Vec<Question>, String> {
    let conn = init_db(&app_handle)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, part, sub_part, audio_path, image_path, text, pack_id, pack_order, response_time, active 
         FROM questions WHERE active = 1 ORDER BY part, sub_part, id"
    ).map_err(|e| e.to_string())?;
    
    let questions = stmt
        .query_map([], |row| {
            Ok(Question {
                id: row.get(0)?,
                part: row.get(1)?,
                sub_part: row.get(2)?,
                audio_path: row.get(3)?,
                image_path: row.get(4)?,
                text: row.get(5)?,
                pack_id: row.get(6)?,
                pack_order: row.get(7)?,
                response_time: row.get(8)?,
                active: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    
    Ok(questions)
}

#[tauri::command]
pub async fn delete_question(
    app_handle: AppHandle,
    question_id: i64,
) -> Result<String, String> {
    let conn = init_db(&app_handle)?;
    
    conn.execute(
        "UPDATE questions SET active = 0 WHERE id = ?",
        [question_id],
    ).map_err(|e| e.to_string())?;
    
    println!("Question {} deactivated successfully", question_id);
    Ok(format!("Question {} deactivated", question_id))
}

#[tauri::command]
pub async fn rate_response(
    app_handle: AppHandle,
    response_id: i64,
    fluency: i32,
    lexical: i32,
    grammar: i32,
    pronunciation: i32,
    comment: String,
) -> Result<(), String> {
    let conn = init_db(&app_handle)?;
    
    conn.execute(
        "INSERT OR REPLACE INTO ratings (response_id, fluency, lexical, grammar, pronunciation, comment) 
         VALUES (?, ?, ?, ?, ?, ?)",
        params![response_id, fluency, lexical, grammar, pronunciation, comment],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn save_image_file(
    app_handle: AppHandle,
    filename: String,
    image_data: Vec<u8>,
) -> Result<String, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let image_dir = app_dir.join("images");
    fs::create_dir_all(&image_dir).map_err(|e| e.to_string())?;

    let safe_filename = unique_filename(&image_dir, &filename)?;
    let image_file_path = image_dir.join(&safe_filename);

    fs::write(&image_file_path, &image_data).map_err(|e| {
        let err_msg = format!("Failed to write image file: {}", e);
        println!("Error: {}", err_msg);
        err_msg
    })?;

    Ok(safe_filename)
}

#[tauri::command]
pub async fn get_image_file(
    app_handle: AppHandle,
    filename: String,
) -> Result<Vec<u8>, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let image_file_path = app_dir.join("images").join(&filename);

    let data = fs::read(&image_file_path)
        .map_err(|e| {
            let err_msg = format!("Failed to read image file {}: {}", filename, e);
            println!("Error: {}", err_msg);
            err_msg
        })?;

    Ok(data)
}

#[tauri::command]
pub async fn get_recordings(
    app_handle: AppHandle,
) -> Result<Vec<Recording>, String> {
    let conn = init_db(&app_handle)?;

    let mut stmt = conn.prepare(
        "SELECT responses.id,
                responses.attempt_id,
                attempts.student_name,
                attempts.started_at,
                responses.question_id,
                questions.part,
                questions.sub_part,
                questions.text,
                responses.recorded_at,
                responses.duration
         FROM responses
         JOIN attempts ON responses.attempt_id = attempts.id
         JOIN questions ON responses.question_id = questions.id
         ORDER BY attempts.started_at DESC, responses.recorded_at DESC"
    ).map_err(|e| e.to_string())?;

    let recordings = stmt
        .query_map([], |row| {
            Ok(Recording {
                id: row.get(0)?,
                attempt_id: row.get(1)?,
                student_name: row.get(2)?,
                attempt_started_at: row.get(3)?,
                question_id: row.get(4)?,
                part: row.get(5)?,
                sub_part: row.get(6)?,
                question_text: row.get(7)?,
                recorded_at: row.get(8)?,
                duration: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(recordings)
}

#[tauri::command]
pub async fn get_response_audio(
    app_handle: AppHandle,
    response_id: i64,
) -> Result<Vec<u8>, String> {
    let conn = init_db(&app_handle)?;

    let audio_path: String = conn.query_row(
        "SELECT audio_path FROM responses WHERE id = ?",
        params![response_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let data = fs::read(&audio_path)
        .map_err(|e| {
            let err_msg = format!("Failed to read response audio: {}", e);
            println!("Error: {}", err_msg);
            err_msg
        })?;

    Ok(data)
}

#[tauri::command]
pub async fn delete_response(
    app_handle: AppHandle,
    response_id: i64,
) -> Result<(), String> {
    let conn = init_db(&app_handle)?;

    let audio_path: Option<String> = conn
        .query_row(
            "SELECT audio_path FROM responses WHERE id = ?",
            params![response_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(path) = audio_path {
        let remove_result = fs::remove_file(&path);
        if let Err(error) = remove_result {
            if error.kind() != std::io::ErrorKind::NotFound {
                return Err(format!("Failed to delete audio file: {}", error));
            }
        }
    }

    conn.execute(
        "DELETE FROM ratings WHERE response_id = ?",
        params![response_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM responses WHERE id = ?",
        params![response_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_attempt(
    app_handle: AppHandle,
    attempt_id: i64,
) -> Result<(), String> {
    let conn = init_db(&app_handle)?;

    let mut stmt = conn
        .prepare("SELECT id, audio_path FROM responses WHERE attempt_id = ?")
        .map_err(|e| e.to_string())?;

    let response_rows = stmt
        .query_map(params![attempt_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut response_ids = Vec::new();
    for row in response_rows {
        let (response_id, audio_path) = row.map_err(|e| e.to_string())?;
        response_ids.push(response_id);
        if !audio_path.is_empty() {
            let remove_result = fs::remove_file(&audio_path);
            if let Err(error) = remove_result {
                if error.kind() != std::io::ErrorKind::NotFound {
                    return Err(format!("Failed to delete audio file: {}", error));
                }
            }
        }
    }

    if !response_ids.is_empty() {
        let placeholders = response_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        for id in &response_ids {
            params.push(id);
        }
        conn.execute(
            &format!("DELETE FROM ratings WHERE response_id IN ({})", placeholders),
            params.as_slice(),
        )
        .map_err(|e| e.to_string())?;
    }

    conn.execute(
        "DELETE FROM responses WHERE attempt_id = ?",
        params![attempt_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM attempts WHERE id = ?",
        params![attempt_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
