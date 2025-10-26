import { invoke } from '@tauri-apps/api/core';

/**
 * OCR recognition result for character level
 */
export interface LevelResult {
  /** Parsed level number (1-300) */
  level: number;
  /** Raw OCR text output */
  raw_text: string;
}

/**
 * OCR recognition result for experience points
 */
export interface ExpResult {
  /** Absolute EXP value */
  absolute: number;
  /** EXP percentage to next level (0.0-99.99) */
  percentage: number;
  /** Raw OCR text output */
  raw_text: string;
}

/**
 * OCR recognition result for map name
 */
export interface MapResult {
  /** Parsed map name (Korean text) */
  map_name: string;
  /** Raw OCR text output */
  raw_text: string;
}

/**
 * Recognize character level from image
 * @param imageBase64 Base64-encoded PNG image
 * @returns Level recognition result
 */
export async function recognizeLevel(imageBase64: string): Promise<LevelResult> {
  return await invoke<LevelResult>('recognize_level', { imageBase64 });
}

/**
 * Recognize experience points from image
 * @param imageBase64 Base64-encoded PNG image
 * @returns EXP recognition result
 */
export async function recognizeExp(imageBase64: string): Promise<ExpResult> {
  return await invoke<ExpResult>('recognize_exp', { imageBase64 });
}

/**
 * Recognize map name from image (Korean text)
 * @param imageBase64 Base64-encoded PNG image
 * @returns Map recognition result
 */
export async function recognizeMap(imageBase64: string): Promise<MapResult> {
  return await invoke<MapResult>('recognize_map', { imageBase64 });
}

/**
 * Recognize HP potion count from inventory image
 * @param imageBase64 Base64-encoded PNG image
 * @returns HP potion count
 */
export async function recognizeHpPotionCount(imageBase64: string): Promise<number> {
  return await invoke<number>('recognize_hp_potion_count', { imageBase64 });
}

/**
 * Recognize MP potion count from inventory image
 * @param imageBase64 Base64-encoded PNG image
 * @returns MP potion count
 */
export async function recognizeMpPotionCount(imageBase64: string): Promise<number> {
  return await invoke<number>('recognize_mp_potion_count', { imageBase64 });
}

/**
 * Check OCR server health status
 * @returns True if OCR server is healthy, false otherwise
 */
export async function checkOcrHealth(): Promise<boolean> {
  try {
    return await invoke<boolean>('check_ocr_health');
  } catch (error) {
    console.error('Health check failed:', error);
    return false;
  }
}

/**
 * Helper: Convert image data URL to base64 string
 * @param dataUrl Image data URL (e.g., from canvas.toDataURL())
 * @returns Base64-encoded image data without prefix
 */
export function dataUrlToBase64(dataUrl: string): string {
  // Remove data URL prefix (e.g., "data:image/png;base64,")
  return dataUrl.split(',')[1];
}

/**
 * Helper: Convert Blob to base64 string
 * @param blob Image blob
 * @returns Promise resolving to base64-encoded image data
 */
export async function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const dataUrl = reader.result as string;
      resolve(dataUrlToBase64(dataUrl));
    };
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}
