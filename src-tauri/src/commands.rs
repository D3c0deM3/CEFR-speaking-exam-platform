use serde::{Deserialize, Serialize};
use rusqlite::{Connection, params};
use tauri::{AppHandle, Manager};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Question {
    pub id: i64,
    pub part: i32,
    pub sub_part: i32,
    pub audio_path: String,
    pub image_path: String,
    pub text: String,
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
        );"
    ).map_err(|e| e.to_string())?;
    
    ensure_column(&conn, "questions", "sub_part", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(&conn, "questions", "image_path", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(&conn, "questions", "text", "TEXT NOT NULL DEFAULT ''")?;
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
        "SELECT id, part, sub_part, audio_path, image_path, text, response_time, active 
         FROM questions 
         WHERE part = ? AND active = 1 {} {}
         ORDER BY RANDOM() 
         LIMIT ?",
        sub_part_clause, exclude_clause
    );
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    
    let map_question = |row: &rusqlite::Row| {
        Ok(Question {
            id: row.get(0)?,
            part: row.get(1)?,
            sub_part: row.get(2)?,
            audio_path: row.get(3)?,
            image_path: row.get(4)?,
            text: row.get(5)?,
            response_time: row.get(6)?,
            active: row.get(7)?,
        })
    };

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
    
    let audio_file_path = app_dir.join("audio").join(&filename);
    println!("Saving audio file to: {:?}, size: {} bytes", audio_file_path, audio_data.len());
    
    fs::create_dir_all(audio_file_path.parent().unwrap()).map_err(|e| e.to_string())?;
    
    fs::write(&audio_file_path, &audio_data).map_err(|e| {
        let err_msg = format!("Failed to write audio file: {}", e);
        println!("Error: {}", err_msg);
        err_msg
    })?;
    
    println!("Audio file saved successfully: {}", filename);
    Ok(filename)
}

#[tauri::command]
pub async fn get_audio_file(
    app_handle: AppHandle,
    filename: String,
) -> Result<Vec<u8>, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    let audio_file_path = app_dir.join("audio").join(&filename);
    println!("Reading audio file from: {:?}", audio_file_path);
    
    let data = fs::read(&audio_file_path)
        .map_err(|e| {
            let err_msg = format!("Failed to read audio file {}: {}", filename, e);
            println!("Error: {}", err_msg);
            err_msg
        })?;
    
    println!("Audio file read successfully: {} bytes", data.len());
    Ok(data)
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
) -> Result<i64, String> {
    let conn = init_db(&app_handle)?;
    
    // Use empty string if no audio path provided
    let audio = audio_path.unwrap_or_default();
    let image = image_path.unwrap_or_default();
    let question_text = text.unwrap_or_default();
    let sub_part_value = sub_part.unwrap_or(0);

    if part == 1 && sub_part_value == 2 && image.is_empty() {
        return Err("Image is required for Part 1.2 questions".to_string());
    }
    
    conn.execute(
        "INSERT INTO questions (part, sub_part, audio_path, image_path, text, response_time, active) 
         VALUES (?, ?, ?, ?, ?, ?, 1)",
        params![part, sub_part_value, audio, image, question_text, response_time],
    ).map_err(|e| e.to_string())?;
    
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub async fn get_questions(
    app_handle: AppHandle,
) -> Result<Vec<Question>, String> {
    let conn = init_db(&app_handle)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, part, sub_part, audio_path, image_path, text, response_time, active 
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
                response_time: row.get(6)?,
                active: row.get(7)?,
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

    let image_file_path = app_dir.join("images").join(&filename);
    fs::create_dir_all(image_file_path.parent().unwrap()).map_err(|e| e.to_string())?;

    fs::write(&image_file_path, &image_data).map_err(|e| {
        let err_msg = format!("Failed to write image file: {}", e);
        println!("Error: {}", err_msg);
        err_msg
    })?;

    Ok(filename)
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
