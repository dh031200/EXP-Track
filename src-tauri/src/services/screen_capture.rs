use crate::models::roi::Roi;
use image::DynamicImage;
use xcap::Monitor;

/// Thread-safe wrapper for xcap::Monitor
///
/// SAFETY: This wrapper implements Send and Sync for Monitor, which is safe because:
/// 1. Monitor is essentially a handle to OS display resources
/// 2. On Windows, HMONITOR handles are thread-safe at the OS level
/// 3. All xcap operations internally handle synchronization
/// 4. We only use Monitor for read-only capture operations
#[derive(Clone)]
struct SendSyncMonitor(Monitor);

// SAFETY: Monitor handles are thread-safe at the OS level.
// The underlying HMONITOR (Windows) or equivalent handles on other platforms
// can be safely sent between threads.
unsafe impl Send for SendSyncMonitor {}

// SAFETY: Monitor operations through xcap are internally synchronized
// and the OS display resources are inherently shareable across threads.
unsafe impl Sync for SendSyncMonitor {}

/// Screen capture service using xcap
pub struct ScreenCapture {
    monitor: SendSyncMonitor,
    scale_factor: f64,
}

impl ScreenCapture {
    /// Create a new screen capture instance using the primary monitor
    pub fn new() -> Result<Self, String> {
        let monitor = Monitor::all()
            .map_err(|e| format!("Failed to get monitors: {}", e))?
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .ok_or("No primary monitor found")?;

        // xcap returns physical pixels, so we need to detect the scale factor
        // On macOS Retina, the scale factor is typically 2.0
        let scale_factor = monitor.scale_factor().unwrap_or(1.0) as f64;

        #[cfg(debug_assertions)]
        {
            let physical_w = monitor.width().unwrap_or(0);
            let physical_h = monitor.height().unwrap_or(0);
            println!("🖥️  Screen Capture Initialized:");
            println!("  Scale Factor: {}", scale_factor);
            println!("  Physical Size: {}x{}", physical_w, physical_h);
            println!("  Logical Size: {}x{}", 
                (physical_w as f64 / scale_factor) as u32,
                (physical_h as f64 / scale_factor) as u32
            );
        }

        Ok(Self {
            monitor: SendSyncMonitor(monitor),
            scale_factor
        })
    }

    /// Create screen capture for a specific monitor by index
    pub fn with_monitor(monitor_index: usize) -> Result<Self, String> {
        let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

        let monitor = monitors
            .get(monitor_index)
            .ok_or(format!("Monitor index {} not found", monitor_index))?
            .clone();

        let scale_factor = monitor.scale_factor().unwrap_or(1.0) as f64;

        Ok(Self {
            monitor: SendSyncMonitor(monitor),
            scale_factor
        })
    }

    /// Capture a specific region of the screen
    /// ROI coordinates are in logical pixels, automatically converted to physical pixels
    pub fn capture_region(&self, roi: &Roi) -> Result<DynamicImage, String> {
        let rgba_image = self
            .monitor.0
            .capture_image()
            .map_err(|e| format!("Failed to capture screen: {}", e))?;

        // Convert RgbaImage to DynamicImage
        let image = DynamicImage::ImageRgba8(rgba_image);

        // Apply scale factor to convert logical coordinates to physical pixels
        // On 125% scale: logical 100x100 → physical 125x125
        // On macOS Retina (2.0): logical 100x100 → physical 200x200
        let physical_x = (roi.x as f64 * self.scale_factor) as u32;
        let physical_y = (roi.y as f64 * self.scale_factor) as u32;
        let physical_width = (roi.width as f64 * self.scale_factor) as u32;
        let physical_height = (roi.height as f64 * self.scale_factor) as u32;

        #[cfg(debug_assertions)]
        {
            println!("🎯 ROI Capture Debug:");
            println!("  Scale Factor: {}", self.scale_factor);
            println!("  Logical ROI: x={}, y={}, w={}, h={}", roi.x, roi.y, roi.width, roi.height);
            println!("  Physical ROI: x={}, y={}, w={}, h={}", physical_x, physical_y, physical_width, physical_height);
            println!("  Full Image: {}x{}", image.width(), image.height());
        }

        // Crop to ROI (with bounds checking)
        let cropped = image.crop_imm(
            physical_x,
            physical_y,
            physical_width.min(image.width().saturating_sub(physical_x)),
            physical_height.min(image.height().saturating_sub(physical_y)),
        );

        Ok(cropped)
    }

    /// Capture entire screen
    pub fn capture_full(&self) -> Result<DynamicImage, String> {
        let rgba_image = self
            .monitor.0
            .capture_image()
            .map_err(|e| format!("Failed to capture screen: {}", e))?;

        Ok(DynamicImage::ImageRgba8(rgba_image))
    }

    /// Get monitor dimensions in logical coordinates
    /// Returns logical size (e.g., 1920x1080) even on HiDPI displays
    pub fn get_dimensions(&self) -> Result<(u32, u32), String> {
        let physical_width = self
            .monitor.0
            .width()
            .map_err(|e| format!("Failed to get width: {}", e))?;
        let physical_height = self
            .monitor.0
            .height()
            .map_err(|e| format!("Failed to get height: {}", e))?;

        // Convert physical pixels to logical coordinates
        // On 125% scale: physical 2400x1350 → logical 1920x1080
        let logical_width = (physical_width as f64 / self.scale_factor) as u32;
        let logical_height = (physical_height as f64 / self.scale_factor) as u32;

        #[cfg(debug_assertions)]
        {
            println!("📐 Screen Dimensions Debug:");
            println!("  Scale Factor: {}", self.scale_factor);
            println!("  Physical: {}x{}", physical_width, physical_height);
            println!("  Logical: {}x{}", logical_width, logical_height);
        }

        Ok((logical_width, logical_height))
    }

    /// Convert image to PNG bytes for transmission
    pub fn image_to_png_bytes(image: &DynamicImage) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode image: {}", e))?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_capture_creation() {
        let result = ScreenCapture::new();
        // This might fail in CI without display
        if result.is_err() {
            println!("Skipping test - no display available");
            return;
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_dimensions() {
        let capture = match ScreenCapture::new() {
            Ok(c) => c,
            Err(_) => {
                println!("Skipping test - no display available");
                return;
            }
        };

        let result = capture.get_dimensions();
        assert!(result.is_ok());

        let (width, height) = result.unwrap();
        assert!(width > 0);
        assert!(height > 0);
        println!("Monitor dimensions: {}x{}", width, height);
    }

    #[test]
    fn test_capture_full_screen() {
        let capture = match ScreenCapture::new() {
            Ok(c) => c,
            Err(_) => {
                println!("Skipping test - no display available");
                return;
            }
        };

        let result = capture.capture_full();
        assert!(result.is_ok());

        let image = result.unwrap();
        let (logical_width, logical_height) = capture.get_dimensions().unwrap();

        // Image dimensions may be scaled on HiDPI displays
        // Just verify we got a valid image with reasonable dimensions
        assert!(image.width() > 0);
        assert!(image.height() > 0);

        // Image should be at least as large as logical dimensions
        // (could be 2x or more on Retina/HiDPI displays)
        assert!(image.width() >= logical_width);
        assert!(image.height() >= logical_height);
    }

    #[test]
    fn test_capture_region() {
        let capture = match ScreenCapture::new() {
            Ok(c) => c,
            Err(_) => {
                println!("Skipping test - no display available");
                return;
            }
        };

        // Capture a 200x150 region from top-left corner (logical coordinates)
        let roi = Roi::new(0, 0, 200, 150);
        let result = capture.capture_region(&roi);

        assert!(result.is_ok());

        let image = result.unwrap();
        // Physical size may differ from logical size on HiDPI displays
        // Just verify we got a valid image with reasonable dimensions
        assert!(image.width() > 0);
        assert!(image.height() > 0);
    }

    #[test]
    fn test_capture_region_bounds_check() {
        let capture = match ScreenCapture::new() {
            Ok(c) => c,
            Err(_) => {
                println!("Skipping test - no display available");
                return;
            }
        };

        let (mon_width, mon_height) = capture.get_dimensions().unwrap();

        // ROI larger than screen - should be clamped
        let roi = Roi::new(
            (mon_width - 50) as i32,
            (mon_height - 50) as i32,
            200,
            150,
        );

        let result = capture.capture_region(&roi);
        assert!(result.is_ok());

        let image = result.unwrap();
        // Should be clamped to available space
        assert!(image.width() <= 200);
        assert!(image.height() <= 150);
    }

    #[test]
    fn test_image_to_png_bytes() {
        let capture = match ScreenCapture::new() {
            Ok(c) => c,
            Err(_) => {
                println!("Skipping test - no display available");
                return;
            }
        };

        let roi = Roi::new(0, 0, 100, 100);
        let image = capture.capture_region(&roi).unwrap();

        let png_bytes = ScreenCapture::image_to_png_bytes(&image);
        assert!(png_bytes.is_ok());

        let bytes = png_bytes.unwrap();
        assert!(!bytes.is_empty());

        // PNG signature check
        assert_eq!(&bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
}
