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
    // Load existing records from file
    match load_sessions_from_file() {
        Ok(records) => {
            // If no records exist, create dummy data for testing
            if records.is_empty() {
                std::sync::Mutex::new(create_dummy_sessions())
            } else {
                std::sync::Mutex::new(records)
            }
        },
        Err(_) => std::sync::Mutex::new(create_dummy_sessions()),
    }
}

fn create_dummy_sessions() -> Vec<SessionRecord> {
    use chrono::{Utc, Duration, TimeZone};
    
    let now = Utc::now();
    
    let ts1 = (now - Duration::hours(48)).timestamp_millis();
    let ts2 = (now - Duration::hours(36)).timestamp_millis();
    let ts3 = (now - Duration::hours(24)).timestamp_millis();
    let ts4 = (now - Duration::hours(18)).timestamp_millis();
    let ts5 = (now - Duration::hours(12)).timestamp_millis();
    let ts6 = (now - Duration::hours(6)).timestamp_millis();
    let ts7 = (now - Duration::hours(3)).timestamp_millis();
    let ts8 = (now - Duration::minutes(45)).timestamp_millis();
    
    vec![
        SessionRecord {
            id: "dummy_1".to_string(),
            title: format_timestamp_to_title(ts1),
            timestamp: ts1,
            combat_time: 3600 + 1825, // 1시간 30분 25초
            exp_gained: 125680000, // 1256만 8000
            current_level: 235,
            avg_exp_per_second: 23145.5,
            hp_potions_used: 142,
            mp_potions_used: 89,
        },
        SessionRecord {
            id: "dummy_2".to_string(),
            title: format_timestamp_to_title(ts2),
            timestamp: ts2,
            combat_time: 2700 + 545, // 45분 45초
            exp_gained: 87530000, // 875만 3000
            current_level: 234,
            avg_exp_per_second: 26945.2,
            hp_potions_used: 98,
            mp_potions_used: 67,
        },
        SessionRecord {
            id: "dummy_3".to_string(),
            title: format_timestamp_to_title(ts3),
            timestamp: ts3,
            combat_time: 4200 + 920, // 1시간 10분 20초
            exp_gained: 156890000, // 1억 5689만
            current_level: 236,
            avg_exp_per_second: 30654.8,
            hp_potions_used: 187,
            mp_potions_used: 124,
        },
        SessionRecord {
            id: "dummy_4".to_string(),
            title: format_timestamp_to_title(ts4),
            timestamp: ts4,
            combat_time: 1800 + 315, // 30분 15초
            exp_gained: 45670000, // 4567만
            current_level: 233,
            avg_exp_per_second: 21584.3,
            hp_potions_used: 54,
            mp_potions_used: 38,
        },
        SessionRecord {
            id: "dummy_5".to_string(),
            title: format_timestamp_to_title(ts5),
            timestamp: ts5,
            combat_time: 5400 + 1245, // 1시간 50분 45초
            exp_gained: 198450000, // 1억 9845만
            current_level: 237,
            avg_exp_per_second: 29876.4,
            hp_potions_used: 223,
            mp_potions_used: 156,
        },
        SessionRecord {
            id: "dummy_6".to_string(),
            title: format_timestamp_to_title(ts6),
            timestamp: ts6,
            combat_time: 2160 + 448, // 36분 8초
            exp_gained: 67890000, // 6789만
            current_level: 235,
            avg_exp_per_second: 26023.7,
            hp_potions_used: 76,
            mp_potions_used: 52,
        },
        SessionRecord {
            id: "dummy_7".to_string(),
            title: format_timestamp_to_title(ts7),
            timestamp: ts7,
            combat_time: 7200 + 1650, // 2시간 27분 30초
            exp_gained: 245670000, // 2억 4567만
            current_level: 238,
            avg_exp_per_second: 27745.2,
            hp_potions_used: 298,
            mp_potions_used: 201,
        },
        SessionRecord {
            id: "dummy_8".to_string(),
            title: format_timestamp_to_title(ts8),
            timestamp: ts8,
            combat_time: 920 + 125, // 15분 25초
            exp_gained: 32450000, // 3245만
            current_level: 234,
            avg_exp_per_second: 31052.6,
            hp_potions_used: 43,
            mp_potions_used: 29,
        },
    ]
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

