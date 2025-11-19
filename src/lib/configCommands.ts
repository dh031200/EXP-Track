import { invoke } from '@tauri-apps/api/core';

/**
 * Potion slot configuration
 */
export interface PotionConfig {
  hp_potion_slot: string;
  mp_potion_slot: string;
}

/**
 * Tracking configuration
 */
export interface TrackingConfig {
  update_interval: number;
  track_meso: boolean;
  auto_start: boolean;
  auto_pause_threshold: number;
}

/**
 * Full Application Configuration
 * (Partial definition for now, expand as needed)
 */
export interface AppConfig {
  potion: PotionConfig;
  tracking: TrackingConfig;
  // Add other sections as needed
  [key: string]: any; 
}

/**
 * Valid inventory slot names
 */
export const VALID_SLOTS = ['shift', 'ins', 'home', 'pup', 'ctrl', 'del', 'end', 'pdn'] as const;
export type SlotName = typeof VALID_SLOTS[number];

/**
 * Get current potion slot configuration
 */
export async function getPotionSlotConfig(): Promise<PotionConfig> {
  return await invoke<PotionConfig>('get_potion_slot_config');
}

/**
 * Set potion slot configuration
 * @param hpSlot - Slot name for HP potion
 * @param mpSlot - Slot name for MP potion
 */
export async function setPotionSlotConfig(hpSlot: string, mpSlot: string): Promise<void> {
  await invoke('set_potion_slot_config', {
    potionConfig: {
      hp_potion_slot: hpSlot,
      mp_potion_slot: mpSlot,
    },
  });
}

/**
 * Load full application configuration
 */
export async function loadAppConfig(): Promise<AppConfig> {
  return await invoke<AppConfig>('load_config');
}

/**
 * Save full application configuration
 */
export async function saveAppConfig(config: AppConfig): Promise<void> {
  return await invoke('save_config', { config });
}
