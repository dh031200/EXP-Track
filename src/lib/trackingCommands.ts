import { invoke } from '@tauri-apps/api/core';
import type { Roi } from './tauri';

/**
 * Tracking statistics from Rust backend
 */
export interface TrackingStats {
  level: number | null;
  exp: number | null;
  percentage: number | null;
  hp: number | null;
  mp: number | null;
  total_exp: number;
  total_percentage: number;
  elapsed_seconds: number;
  exp_per_hour: number;
  percentage_per_hour: number;
  is_tracking: boolean;
  error: string | null;
  hp_potions_used: number;
  mp_potions_used: number;
  hp_potions_per_minute: number;
  mp_potions_per_minute: number;
}

/**
 * Start OCR tracking with 3 parallel tasks (Level, EXP, Inventory all with manual ROI)
 * All regions must be manually selected by the user
 * Potion slot mapping is configured via settings
 */
export async function startOcrTracking(
  levelRoi: Roi,
  expRoi: Roi,
  inventoryRoi: Roi
): Promise<void> {
  await invoke('start_ocr_tracking', {
    levelRoi,
    expRoi,
    inventoryRoi,
  });
}

/**
 * Stop OCR tracking
 */
export async function stopOcrTracking(): Promise<void> {
  await invoke('stop_ocr_tracking');
}

/**
 * Get current tracking statistics
 */
export async function getTrackingStats(): Promise<TrackingStats> {
  return await invoke<TrackingStats>('get_tracking_stats');
}

/**
 * Reset tracking session
 */
export async function resetTracking(): Promise<void> {
  await invoke('reset_tracking');
}
