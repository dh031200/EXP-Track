import { invoke } from '@tauri-apps/api/core';

/**
 * Potion slot configuration
 */
export interface PotionConfig {
  hp_potion_slot: string;
  mp_potion_slot: string;
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
