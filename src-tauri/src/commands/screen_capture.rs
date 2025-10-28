use crate::models::roi::Roi;
use crate::services::screen_capture::ScreenCapture;
use tauri::State;
use std::sync::Mutex;

/// State wrapper for screen capture service
pub type ScreenCaptureState = Mutex<Option<ScreenCapture>>;

/// Initialize screen capture with primary monitor
#[tauri::command]
pub fn init_screen_capture(state: State<ScreenCaptureState>) -> Result<(), String> {
    let capture = ScreenCapture::new()?;
    let mut state_guard = state.inner().lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    *state_guard = Some(capture);
    Ok(())
}

/// Get monitor dimensions (logical width/height)
#[tauri::command]
pub fn get_screen_dimensions(state: State<ScreenCaptureState>) -> Result<(u32, u32), String> {
    let state_guard = state.inner().lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    let capture = state_guard
        .as_ref()
        .ok_or("Screen capture not initialized")?;
    capture.get_dimensions()
}

/// Capture a specific region and return as PNG bytes (base64 encoded)
#[tauri::command]
pub fn capture_region(
    state: State<ScreenCaptureState>,
    roi: Roi,
) -> Result<Vec<u8>, String> {
    let state_guard = state.inner().lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    let capture = state_guard
        .as_ref()
        .ok_or("Screen capture not initialized")?;

    let image = capture.capture_region(&roi)?;
    ScreenCapture::image_to_png_bytes(&image)
}

/// Capture full screen and return as PNG bytes (base64 encoded)
#[tauri::command]
pub fn capture_full_screen(state: State<ScreenCaptureState>) -> Result<Vec<u8>, String> {
    let state_guard = state.inner().lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    let capture = state_guard
        .as_ref()
        .ok_or("Screen capture not initialized")?;

    let image = capture.capture_full()?;
    ScreenCapture::image_to_png_bytes(&image)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to initialize a state directly (not through Tauri command)
    fn init_state() -> Result<ScreenCaptureState, String> {
        let capture = ScreenCapture::new()?;
        let state = ScreenCaptureState::default();
        *state.lock().unwrap() = Some(capture);
        Ok(state)
    }

    #[test]
    fn test_state_initialization() {
        let result = init_state();

        // May fail in CI without display
        if result.is_err() {
            println!("Skipping test - no display available");
            return;
        }

        assert!(result.is_ok());

        // Verify state was set
        let state = result.unwrap();
        let guard = state.lock().unwrap();
        assert!(guard.is_some());
    }

    #[test]
    fn test_uninitialized_state() {
        let state = ScreenCaptureState::default();
        let guard = state.lock().unwrap();
        assert!(guard.is_none());
    }

    #[test]
    fn test_screen_capture_workflow() {
        let state = match init_state() {
            Ok(s) => s,
            Err(_) => {
                println!("Skipping test - no display available");
                return;
            }
        };

        // Test getting dimensions
        {
            let guard = state.lock().unwrap();
            let capture = guard.as_ref().unwrap();
            let result = capture.get_dimensions();
            assert!(result.is_ok());

            let (width, height) = result.unwrap();
            assert!(width > 0);
            assert!(height > 0);
        }

        // Test capturing region
        {
            let guard = state.lock().unwrap();
            let capture = guard.as_ref().unwrap();
            let roi = Roi::new(0, 0, 100, 100);
            let result = capture.capture_region(&roi);
            assert!(result.is_ok());

            let image = result.unwrap();
            let png_bytes = ScreenCapture::image_to_png_bytes(&image);
            assert!(png_bytes.is_ok());

            let bytes = png_bytes.unwrap();
            assert!(!bytes.is_empty());
            assert_eq!(&bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        }

        // Test capturing full screen
        {
            let guard = state.lock().unwrap();
            let capture = guard.as_ref().unwrap();
            let result = capture.capture_full();
            assert!(result.is_ok());

            let image = result.unwrap();
            let png_bytes = ScreenCapture::image_to_png_bytes(&image);
            assert!(png_bytes.is_ok());

            let bytes = png_bytes.unwrap();
            assert!(!bytes.is_empty());
            assert_eq!(&bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        }
    }
}
