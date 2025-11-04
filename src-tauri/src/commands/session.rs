use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub title: String,
    pub timestamp: i64,
    pub combat_time: i32,
    pub exp_gained: i64,
    pub current_level: i32,
    pub avg_exp_per_second: f64,
    pub hp_potions_used: i32,
    pub mp_potions_used: i32,
}

pub type SessionRecordsState = std::sync::Mutex<Vec<SessionRecord>>;

pub fn init_session_records() -> SessionRecordsState {
    match load_sessions_from_file() {
        Ok(records) => std::sync::Mutex::new(records),
        Err(_) => std::sync::Mutex::new(Vec::new()),
    }
}

fn format_timestamp_to_title(timestamp_millis: i64) -> String {
    use chrono::{Local, TimeZone};
    
    let datetime = Local.timestamp_millis_opt(timestamp_millis).unwrap();
    datetime.format("%Y년 %m월 %d일 %H:%M 전투").to_string()
}

fn get_sessions_file_path() -> Result<PathBuf, String> {
    let app_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("exp-tracker");
    
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app directory: {}", e))?;
    
    Ok(app_dir.join("session_records.json"))
}

fn load_sessions_from_file() -> Result<Vec<SessionRecord>, String> {
    let file_path = get_sessions_file_path()?;
    
    if !file_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read sessions file: {}", e))?;
    
    let records: Vec<SessionRecord> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse sessions: {}", e))?;
    
    Ok(records)
}

fn save_sessions_to_file(records: &[SessionRecord]) -> Result<(), String> {
    let file_path = get_sessions_file_path()?;
    
    let content = serde_json::to_string_pretty(records)
        .map_err(|e| format!("Failed to serialize sessions: {}", e))?;
    
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write sessions file: {}", e))?;
    
    Ok(())
}

/// Get all session records
#[tauri::command]
pub fn get_session_records(state: State<SessionRecordsState>) -> Result<Vec<SessionRecord>, String> {
    let records = state.lock()
        .map_err(|e| format!("Failed to lock session state: {}", e))?;
    
    Ok(records.clone())
}

/// Save a new session record
#[tauri::command]
pub fn save_session_record(
    state: State<SessionRecordsState>,
    record: SessionRecord,
) -> Result<(), String> {
    let mut records = state.lock()
        .map_err(|e| format!("Failed to lock session state: {}", e))?;
    
    // Add new record at the beginning (most recent first)
    records.insert(0, record);
    
    // Save to file
    save_sessions_to_file(&records)?;
    
    Ok(())
}

/// Delete a session record by ID
#[tauri::command]
pub fn delete_session_record(
    state: State<SessionRecordsState>,
    id: String,
) -> Result<(), String> {
    let mut records = state.lock()
        .map_err(|e| format!("Failed to lock session state: {}", e))?;
    
    // Remove record with matching ID
    records.retain(|r| r.id != id);
    
    // Save to file
    save_sessions_to_file(&records)?;
    
    Ok(())
}

/// Update the title of a session record
#[tauri::command]
pub fn update_session_title(
    state: State<SessionRecordsState>,
    id: String,
    new_title: String,
) -> Result<(), String> {
    let mut records = state.lock()
        .map_err(|e| format!("Failed to lock session state: {}", e))?;
    
    // Find and update the record with matching ID
    if let Some(record) = records.iter_mut().find(|r| r.id == id) {
        record.title = new_title;
    } else {
        return Err(format!("Session record with id '{}' not found", id));
    }
    
    // Save to file
    save_sessions_to_file(&records)?;
    
    Ok(())
}

