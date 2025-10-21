import { invoke } from '@tauri-apps/api/core';
import type { Roi } from './tauri';

/**
 * ROI type identifier matching Rust enum
 */
export type RoiType = 'level' | 'exp' | 'meso';

/**
 * Save ROI configuration for a specific type
 * @param roiType Type of ROI (level, exp, or meso)
 * @param roi ROI coordinates
 */
export async function saveRoi(roiType: RoiType, roi: Roi): Promise<void> {
  await invoke('save_roi', { roiType, roi });
}

/**
 * Load ROI configuration for a specific type
 * @param roiType Type of ROI (level, exp, or meso)
 * @returns ROI if configured, null otherwise
 */
export async function loadRoi(roiType: RoiType): Promise<Roi | null> {
  return await invoke<Roi | null>('load_roi', { roiType });
}

/**
 * Get all configured ROIs
 * @returns Object with level, exp, and meso ROIs
 */
export async function getAllRois(): Promise<{
  level: Roi | null;
  exp: Roi | null;
  meso: Roi | null;
}> {
  return await invoke('get_all_rois');
}

/**
 * Clear (remove) ROI configuration for a specific type
 * @param roiType Type of ROI to clear
 */
export async function clearRoi(roiType: RoiType): Promise<void> {
  await invoke('clear_roi', { roiType });
}

/**
 * Get the config file path
 * @returns Absolute path to the config file
 */
export async function getConfigPath(): Promise<string> {
  return await invoke<string>('get_config_path');
}
