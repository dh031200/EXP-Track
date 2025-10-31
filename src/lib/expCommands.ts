import { invoke } from '@tauri-apps/api/core';

/**
 * EXP tracking statistics
 */
export interface ExpStats {
  /** Current level (nullable during initial tracking) */
  level: number | null;
  /** Current EXP value (nullable during initial tracking) */
  exp: number | null;
  /** Current percentage to next level (nullable during initial tracking) */
  percentage: number | null;
  /** Current HP potion count (nullable during initial tracking) */
  hp_potion_count: number | null;
  /** Current MP potion count (nullable during initial tracking) */
  mp_potion_count: number | null;
  /** Total EXP gained during session */
  total_exp: number;
  /** Total percentage gained (including level ups) */
  total_percentage: number;
  /** Elapsed time in seconds */
  elapsed_seconds: number;
  /** EXP gained per hour */
  exp_per_hour: number;
  /** Percentage gained per hour */
  percentage_per_hour: number;
  /** Is currently tracking */
  is_tracking: boolean;
  /** Error message if any */
  error: string | null;
  /** Total HP potions used */
  hp_potions_used: number;
  /** Total MP potions used */
  mp_potions_used: number;
  /** HP potions consumed per minute */
  hp_potions_per_minute: number;
  /** MP potions consumed per minute */
  mp_potions_per_minute: number;
  /** OCR server health status */
  ocr_server_healthy: boolean;
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
 * Format number in Korean units (만)
 * e.g., 576558 -> "57만 6558", 8001234 -> "800만 1234"
 */
export function formatKoreanNumber(num: number): string {
  if (num < 10000) {
    return num.toLocaleString('ko-KR');
  }
  
  const man = Math.floor(num / 10000);
  const remainder = num % 10000;
  
  if (remainder === 0) {
    return `${man.toLocaleString('ko-KR')}만`;
  }
  
  return `${man.toLocaleString('ko-KR')}만 ${remainder.toLocaleString('ko-KR')}`;
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

/**
 * Format number with K/M suffix for compact display
 * e.g., 500 -> "500", 1500 -> "1.5K", 1500000 -> "1.5M"
 */
export function formatCompact(num: number): string {
  if (num < 1000) {
    return num.toString();
  } else if (num < 1000000) {
    const k = num / 1000;
    return k % 1 === 0 ? `${k}K` : `${k.toFixed(1)}K`;
  } else {
    const m = num / 1000000;
    return m % 1 === 0 ? `${m}M` : `${m.toFixed(1)}M`;
  }
}