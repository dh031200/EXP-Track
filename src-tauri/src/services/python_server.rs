use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

/// Python OCR Server Manager
/// Handles automatic start/stop of the Python FastAPI server
pub struct PythonServerManager {
    process: Option<Child>,
    base_url: String,
}

impl PythonServerManager {
    /// Create a new server manager
    pub fn new() -> Self {
        Self {
            process: None,
            base_url: "http://127.0.0.1:39835".to_string(),
        }
    }

    /// Start the Python OCR server using bundled binary
    pub async fn start(&mut self) -> Result<(), String> {
        #[cfg(debug_assertions)]
        println!("🚀 Starting Python OCR server...");

        // Check if already running
        if self.is_server_running().await {
            #[cfg(debug_assertions)]
            println!("✅ Python OCR server already running");
            return Ok(());
        }

        // Start bundled server
        let child = self.start_server()?;
        self.process = Some(child);

        // Wait for server to be ready
        self.wait_for_ready().await?;

        #[cfg(debug_assertions)]
        println!("✅ Python OCR server started successfully");

        Ok(())
    }

    /// Start server using bundled binary (onedir mode)
    fn start_server(&self) -> Result<Child, String> {
        // Get the directory where the executable is located
        let exe_dir = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?
            .parent()
            .ok_or("Failed to get exe parent dir")?
            .to_path_buf();

        // Look for ocr_server directory in resources (for bundled app)
        let server_dir = exe_dir.join("../Resources/resources/ocr_server");
        let server_bin = server_dir.join("ocr_server");

        // Fallback: look in src-tauri/resources (for development)
        let (server_dir, server_bin) = if server_bin.exists() {
            (server_dir, server_bin)
        } else {
            let dev_dir = exe_dir.join("resources/ocr_server");
            let dev_bin = dev_dir.join("ocr_server");
            (dev_dir, dev_bin)
        };

        if !server_bin.exists() {
            return Err(format!(
                "OCR server binary not found at: {:?}\n\n\
                Please build it first:\n\
                  ./scripts/build_python_server.sh",
                server_bin
            ));
        }

        #[cfg(debug_assertions)]
        println!("📍 Server directory: {:?}", server_dir);
        #[cfg(debug_assertions)]
        println!("📍 Server binary: {:?}", server_bin);

        Command::new(server_bin)
            .current_dir(server_dir)
            .spawn()
            .map_err(|e| format!("Failed to start server: {}", e))
    }

    /// Check if server is running by hitting health endpoint
    async fn is_server_running(&self) -> bool {
        let url = format!("{}/health", self.base_url);

        match reqwest::get(&url).await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Wait for server to be ready (max 30 seconds)
    async fn wait_for_ready(&self) -> Result<(), String> {
        // Initial wait for server process to boot (PyInstaller takes ~2-3 seconds)
        sleep(Duration::from_millis(2000)).await;

        let max_attempts = 60; // 60 attempts * 500ms = 30 seconds
        let delay = Duration::from_millis(500);

        for attempt in 1..=max_attempts {
            if self.is_server_running().await {
                #[cfg(debug_assertions)]
                println!("✅ Server ready after {} attempts", attempt);
                return Ok(());
            }

            #[cfg(debug_assertions)]
            if attempt % 3 == 0 {
                println!("⏳ Waiting for server... (attempt {}/{})", attempt, max_attempts);
            }

            sleep(delay).await;
        }

        Err("Server failed to start within 30 seconds. Check if port 39835 is available.".to_string())
    }

    /// Stop the server gracefully via shutdown endpoint, fallback to kill
    pub fn stop(&mut self) {
        #[cfg(debug_assertions)]
        println!("⏹️  Stopping Python OCR server...");

        // Try graceful shutdown via HTTP endpoint first
        let rt = tokio::runtime::Runtime::new().unwrap();
        let shutdown_url = format!("{}/shutdown", self.base_url);
        
        let graceful_shutdown = rt.block_on(async {
            match reqwest::Client::new()
                .post(&shutdown_url)
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    println!("✅ Graceful shutdown signal sent");
                    true
                }
                Err(_) => false,
            }
        });

        if graceful_shutdown {
            // Wait a bit for graceful shutdown
            std::thread::sleep(Duration::from_millis(1000));
        }

        // Fallback: force kill if we have a process handle
        if let Some(mut child) = self.process.take() {
            match child.kill() {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    println!("✅ Python OCR server stopped (forced)");
                }
                Err(e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("❌ Failed to stop server: {}", e);
                }
            }
        } else if graceful_shutdown {
            #[cfg(debug_assertions)]
            println!("✅ Python OCR server stopped (graceful)");
        }
    }

    /// Get the server base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for PythonServerManager {
    fn drop(&mut self) {
        self.stop();
    }
}
