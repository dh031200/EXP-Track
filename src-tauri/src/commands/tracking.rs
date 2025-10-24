use crate::models::roi::Roi;
use crate::services::ocr_tracker::{OcrTracker, TrackingStats};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Global OCR Tracker instance (shared across all commands)
pub struct TrackerState(pub Arc<Mutex<OcrTracker>>);

impl TrackerState {
    pub fn new() -> Result<Self, String> {
        Ok(Self(Arc::new(Mutex::new(OcrTracker::new()?))))
    }
}

/// Start OCR tracking with 4 parallel tasks
#[tauri::command]
pub async fn start_ocr_tracking(
    level_roi: Roi,
    exp_roi: Roi,
    hp_roi: Roi,
    mp_roi: Roi,
    tracker: State<'_, TrackerState>,
) -> Result<(), String> {
    let tracker = tracker.0.lock().await;
    tracker.start_tracking(level_roi, exp_roi, hp_roi, mp_roi).await
}

/// Stop OCR tracking
#[tauri::command]
pub async fn stop_ocr_tracking(tracker: State<'_, TrackerState>) -> Result<(), String> {
    let tracker = tracker.0.lock().await;
    tracker.stop_tracking().await;
    Ok(())
}

/// Get current tracking statistics
#[tauri::command]
pub async fn get_tracking_stats(tracker: State<'_, TrackerState>) -> Result<TrackingStats, String> {
    let tracker = tracker.0.lock().await;
    Ok(tracker.get_stats().await)
}

/// Reset tracking session
#[tauri::command]
pub async fn reset_tracking(tracker: State<'_, TrackerState>) -> Result<(), String> {
    let tracker = tracker.0.lock().await;
    tracker.reset().await;
    Ok(())
}
