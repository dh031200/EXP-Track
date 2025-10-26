import { invoke } from '@tauri-apps/api/core';

/**
 * EXP tracking statistics
 */
export interface ExpStats {
  /** Total EXP gained during session */
  total_exp: number;
  /** Total percentage gained (including level ups) */
  total_percentage: number;
  /** Total meso gained during session */
  total_meso: number;
  /** Elapsed time in seconds */
  elapsed_seconds: number;
  /** EXP gained per hour */
  exp_per_hour: number;
  /** Percentage gained per hour */
  percentage_per_hour: number;
  /** Meso gained per hour */
  meso_per_hour: number;
  /** EXP gained per minute */
  exp_per_minute: number;
  /** Current character level */
  current_level: number;
  /** Starting level of session */
  start_level: number;
  /** Number of levels gained */
  levels_gained: number;
  /** Total HP potions used */
  hp_potions_used: number;
  /** Total MP potions used */
  mp_potions_used: number;
  /** HP potions consumed per minute */
  hp_potions_per_minute: number;
  /** MP potions consumed per minute */
  mp_potions_per_minute: number;
}

/**
 * Start a new EXP tracking session
 * @param level Current character level
 * @param exp Current EXP within level
 * @param percentage Current percentage to next level (0-100)
 * @param meso Current meso amount (optional)
 * @returns Success message
 */
export async function startExpSession(
  level: number,
  exp: number,
  percentage: number,
  meso?: number
): Promise<string> {
  return await invoke<string>('start_exp_session', {
    level,
    exp,
    percentage,
    meso: meso ?? null,
  });
}

/**
 * Add new EXP data and get updated statistics
 * @param level Current character level
 * @param exp Current EXP within level
 * @param percentage Current percentage to next level (0-100)
 * @param meso Current meso amount (optional)
 * @returns Updated EXP statistics
 */
export async function addExpData(
  level: number,
  exp: number,
  percentage: number,
  meso?: number
): Promise<ExpStats> {
  return await invoke<ExpStats>('add_exp_data', {
    level,
    exp,
    percentage,
    meso: meso ?? null,
  });
}

/**
 * Reset the current EXP tracking session
 * @returns Success message
 */
export async function resetExpSession(): Promise<string> {
  return await invoke<string>('reset_exp_session');
}

/**
 * Format seconds as HH:MM:SS
 */
export function formatElapsedTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Format number with thousands separator
 */
export function formatNumber(num: number): string {
  return num.toLocaleString('ko-KR');
}

/**
 * Format percentage with 2 decimal places, ensuring 2 digits before decimal
 * e.g., 5.5 -> "05.50%", 12.34 -> "12.34%", 100 -> "100.00%"
 */
export function formatPercentage(pct: number): string {
  const rounded = pct.toFixed(2);
  const parts = rounded.split('.');
  const intPart = parts[0].padStart(2, '0');
  return `${intPart}.${parts[1]}%`;
}
