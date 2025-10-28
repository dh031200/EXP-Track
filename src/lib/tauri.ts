import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';

export interface Roi {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface WindowState {
  width: number;
  height: number;
  x: number;
  y: number;
}

/**
 * Initialize screen capture with the primary monitor
 */
export async function initScreenCapture(): Promise<void> {
  return invoke('init_screen_capture');
}

/**
 * Get the logical screen dimensions
 */
export async function getScreenDimensions(): Promise<[number, number]> {
  return invoke('get_screen_dimensions');
}

/**
 * Capture a specific region and return as PNG bytes
 */
export async function captureRegion(roi: Roi): Promise<number[]> {
  return invoke('capture_region', { roi });
}

/**
 * Capture full screen and return as PNG bytes
 */
export async function captureFullScreen(): Promise<number[]> {
  return invoke('capture_full_screen');
}

/**
 * Convert PNG bytes to base64 data URL for display
 */
export function bytesToDataUrl(bytes: number[]): string {
  const uint8Array = new Uint8Array(bytes);

  // Convert to base64 in chunks to avoid stack overflow with large images
  let binary = '';
  const chunkSize = 8192;
  for (let i = 0; i < uint8Array.length; i += chunkSize) {
    const chunk = uint8Array.subarray(i, Math.min(i + chunkSize, uint8Array.length));
    binary += String.fromCharCode(...chunk);
  }

  const base64 = btoa(binary);
  return `data:image/png;base64,${base64}`;
}

/**
 * Maximize window to screen size (not fullscreen, just resize)
 * Returns the original window state for restoration
 *
 * DEBUG VERSION: NO ERROR HANDLING - Let errors crash to see real cause
 */
export async function maximizeWindowForROI(): Promise<WindowState> {
  const window = getCurrentWindow();

  // Save current state
  const size = await window.innerSize();
  const position = await window.outerPosition();

  // Convert to logical coordinates for consistency (HiDPI-aware)
  const logicalSize = size.toLogical(await window.scaleFactor());
  const logicalPosition = position.toLogical(await window.scaleFactor());

  const originalState: WindowState = {
    width: logicalSize.width,
    height: logicalSize.height,
    x: logicalPosition.x,
    y: logicalPosition.y,
  };

  // Get screen dimensions
  const [screenWidth, screenHeight] = await getScreenDimensions();

  // Resize to almost full screen (leave small margin for visibility)
  const margin = 0;
  const newX = margin;
  const newY = margin;
  const newWidth = screenWidth - margin * 2;
  const newHeight = screenHeight - margin * 2;

  // Use LogicalPosition for proper HiDPI support on macOS (e.g., 3840x2160 → 1920x1080)
  await window.setPosition(new LogicalPosition(newX, newY));

  // Use LogicalSize for proper HiDPI support on macOS (fills screen correctly)
  await window.setSize(new LogicalSize(newWidth, newHeight));

  return originalState;
}

/**
 * Restore window to its original size and position
 */
export async function restoreWindow(state: WindowState): Promise<void> {
  const window = getCurrentWindow();
  // Use LogicalPosition and LogicalSize for proper HiDPI support
  await window.setPosition(new LogicalPosition(state.x, state.y));
  await window.setSize(new LogicalSize(state.width, state.height));
}

/**
 * Set window to always stay on top for ROI overlay
 */
export async function setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
  const window = getCurrentWindow();
  await window.setAlwaysOnTop(alwaysOnTop);
}
