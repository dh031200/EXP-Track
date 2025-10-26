import { captureRegion, type Roi } from '../lib/tauri';
import {
  recognizeLevel,
  recognizeExp,
  recognizeHpPotionCount,
  recognizeMpPotionCount,
} from '../lib/ocrCommands';
import {
  useLevelStore,
  useExpStore,
  useHpPotionStore,
  useMpPotionStore,
} from '../stores/ocrStore';

/**
 * Convert raw PNG bytes to base64 string
 */
function bytesToBase64(bytes: number[]): string {
  const uint8Array = new Uint8Array(bytes);
  let binary = '';
  const chunkSize = 8192;
  for (let i = 0; i < uint8Array.length; i += chunkSize) {
    const chunk = uint8Array.subarray(i, Math.min(i + chunkSize, uint8Array.length));
    binary += String.fromCharCode(...chunk);
  }
  return btoa(binary);
}

/**
 * Helper to sleep for a given duration
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * OCR Service with independent loops for each recognition type
 * Each OCR operates independently and in parallel
 */
export class OcrService {
  private levelRunning = false;
  private expRunning = false;
  private hpRunning = false;
  private mpRunning = false;

  /**
   * Start LEVEL OCR independent loop
   * Completes OCR → waits 500ms → repeats
   */
  async startLevelLoop(getROI: () => Roi | null): Promise<void> {
    this.levelRunning = true;
    console.log('🚀 LEVEL OCR loop started');

    while (this.levelRunning) {
      const startTime = Date.now();
      const roi = getROI();

      if (!roi) {
        useLevelStore.getState().setError('Level ROI not configured');
        await sleep(500);
        continue;
      }

      try {
        const bytes = await captureRegion(roi);
        const base64 = bytesToBase64(bytes);
        const result = await recognizeLevel(base64);
        const elapsed = Date.now() - startTime;

        useLevelStore.getState().setLevel(result);
        console.log(`✅ LEVEL OCR: ${result.level} (${elapsed}ms)`);
      } catch (err) {
        const elapsed = Date.now() - startTime;
        const error = err instanceof Error ? err.message : String(err);
        useLevelStore.getState().setError(error);
        console.warn(`❌ LEVEL OCR failed: ${error} (${elapsed}ms)`);
      }

      // Wait 500ms after completion
      await sleep(500);
    }

    console.log('⏹️  LEVEL OCR loop stopped');
  }

  /**
   * Start EXP OCR independent loop
   * Completes OCR → waits 500ms → repeats
   */
  async startExpLoop(getROI: () => Roi | null): Promise<void> {
    this.expRunning = true;
    console.log('🚀 EXP OCR loop started');

    while (this.expRunning) {
      const startTime = Date.now();
      const roi = getROI();

      if (!roi) {
        useExpStore.getState().setError('EXP ROI not configured');
        await sleep(500);
        continue;
      }

      try {
        const bytes = await captureRegion(roi);
        const base64 = bytesToBase64(bytes);
        const result = await recognizeExp(base64);
        const elapsed = Date.now() - startTime;

        useExpStore.getState().setExp(result);
        console.log(`✅ EXP OCR: ${result.absolute} [${result.percentage}%] (${elapsed}ms)`);
      } catch (err) {
        const elapsed = Date.now() - startTime;
        const error = err instanceof Error ? err.message : String(err);
        useExpStore.getState().setError(error);
        console.warn(`❌ EXP OCR failed: ${error} (${elapsed}ms)`);
      }

      // Wait 500ms after completion
      await sleep(500);
    }

    console.log('⏹️  EXP OCR loop stopped');
  }

  /**
   * Start HP Potion OCR independent loop
   * Completes OCR → waits 500ms → repeats
   */
  async startHpLoop(getROI: () => Roi | null): Promise<void> {
    this.hpRunning = true;
    console.log('🚀 HP Potion OCR loop started');

    while (this.hpRunning) {
      const startTime = Date.now();
      const roi = getROI();

      if (!roi) {
        useHpPotionStore.getState().setError('HP Potion ROI not configured');
        await sleep(500);
        continue;
      }

      try {
        const bytes = await captureRegion(roi);
        const base64 = bytesToBase64(bytes);
        const result = await recognizeHpPotionCount(base64);
        const elapsed = Date.now() - startTime;

        useHpPotionStore.getState().setHpPotionCount(result);
        console.log(`✅ HP Potion OCR: ${result} (${elapsed}ms)`);
      } catch (err) {
        const elapsed = Date.now() - startTime;
        const error = err instanceof Error ? err.message : String(err);
        useHpPotionStore.getState().setError(error);
        console.warn(`❌ HP Potion OCR failed: ${error} (${elapsed}ms)`);
      }

      // Wait 500ms after completion
      await sleep(500);
    }

    console.log('⏹️  HP Potion OCR loop stopped');
  }

  /**
   * Start MP Potion OCR independent loop
   * Completes OCR → waits 500ms → repeats
   */
  async startMpLoop(getROI: () => Roi | null): Promise<void> {
    this.mpRunning = true;
    console.log('🚀 MP Potion OCR loop started');

    while (this.mpRunning) {
      const startTime = Date.now();
      const roi = getROI();

      if (!roi) {
        useMpPotionStore.getState().setError('MP Potion ROI not configured');
        await sleep(500);
        continue;
      }

      try {
        const bytes = await captureRegion(roi);
        const base64 = bytesToBase64(bytes);
        const result = await recognizeMpPotionCount(base64);
        const elapsed = Date.now() - startTime;

        useMpPotionStore.getState().setMpPotionCount(result);
        console.log(`✅ MP Potion OCR: ${result} (${elapsed}ms)`);
      } catch (err) {
        const elapsed = Date.now() - startTime;
        const error = err instanceof Error ? err.message : String(err);
        useMpPotionStore.getState().setError(error);
        console.warn(`❌ MP Potion OCR failed: ${error} (${elapsed}ms)`);
      }

      // Wait 500ms after completion
      await sleep(500);
    }

    console.log('⏹️  MP Potion OCR loop stopped');
  }

  /**
   * Start all OCR loops in parallel
   * Each loop operates independently
   * IMPORTANT: Return promises to ensure actual parallelism
   */
  startAllLoops(
    getLevelROI: () => Roi | null,
    getExpROI: () => Roi | null,
    getHpROI: () => Roi | null,
    getMpROI: () => Roi | null
  ): void {
    // Start all 4 loops as independent promises (truly parallel)
    Promise.all([
      this.startLevelLoop(getLevelROI),
      this.startExpLoop(getExpROI),
      this.startHpLoop(getHpROI),
      this.startMpLoop(getMpROI),
    ]).catch(err => {
      console.error('❌ OCR loop error:', err);
    });

    console.log('🚀 All OCR loops started (parallel execution)');
  }

  /**
   * Stop all OCR loops
   */
  stopAllLoops(): void {
    this.levelRunning = false;
    this.expRunning = false;
    this.hpRunning = false;
    this.mpRunning = false;

    console.log('⏹️  All OCR loops stopped');
  }

  /**
   * Stop individual loops
   */
  stopLevelLoop(): void {
    this.levelRunning = false;
    console.log('⏹️  Level OCR loop stopped');
  }

  stopExpLoop(): void {
    this.expRunning = false;
    console.log('⏹️  EXP OCR loop stopped');
  }

  stopHpLoop(): void {
    this.hpRunning = false;
    console.log('⏹️  HP OCR loop stopped');
  }

  stopMpLoop(): void {
    this.mpRunning = false;
    console.log('⏹️  MP OCR loop stopped');
  }

  /**
   * Check if any loop is running
   */
  isRunning(): boolean {
    return this.levelRunning || this.expRunning || this.hpRunning || this.mpRunning;
  }

  /**
   * Clear all OCR stores
   */
  clearAllStores(): void {
    useLevelStore.getState().clear();
    useExpStore.getState().clear();
    useHpPotionStore.getState().clear();
    useMpPotionStore.getState().clear();
  }
}

/**
 * Singleton instance of OCR service
 */
export const ocrService = new OcrService();
