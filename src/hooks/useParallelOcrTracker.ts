import { useEffect, useRef, useCallback, useState } from 'react';
import { ocrService } from '../services/ocrService';
import { useRoiStore } from '../stores/roiStore';
import { useLevelStore, useExpStore } from '../stores/ocrStore';
import { startExpSession, addExpData, resetExpSession, type ExpStats } from '../lib/expCommands';

/**
 * Hook for parallel OCR tracking with independent loops
 *
 * Features:
 * - Each OCR (Level, EXP, HP, MP) runs in its own independent loop
 * - Each OCR completes → waits 1 second → repeats
 * - Failures in one OCR don't affect others
 * - All 4 OCRs run in parallel
 * - Integrates with ExpCalculator backend
 */
export function useParallelOcrTracker() {
  const { levelRoi, expRoi, hpRoi, mpRoi } = useRoiStore();

  const isTrackingRef = useRef(false);
  const sessionStartedRef = useRef(false);
  const statsIntervalRef = useRef<number | null>(null);
  const [currentStats, setCurrentStats] = useState<ExpStats | null>(null);

  // Subscribe to level and EXP changes to update ExpCalculator
  const prevLevelRef = useRef<number | null>(null);
  const prevExpRef = useRef<number | null>(null);

  /**
   * Start OCR tracking and ExpCalculator session
   */
  const start = useCallback(async () => {
    if (!levelRoi || !expRoi) {
      console.error('Level and EXP ROI must be configured');
      return;
    }

    isTrackingRef.current = true;
    sessionStartedRef.current = false;

    // Clear previous stores
    ocrService.clearAllStores();

    // Start all OCR loops in parallel
    ocrService.startAllLoops(
      () => levelRoi,
      () => expRoi,
      () => hpRoi,
      () => mpRoi
    );

    console.log('🚀 Parallel OCR tracking started');
  }, [levelRoi, expRoi, hpRoi, mpRoi]);

  /**
   * Stop OCR tracking
   */
  const stop = useCallback(() => {
    isTrackingRef.current = false;
    ocrService.stopAllLoops();

    if (statsIntervalRef.current !== null) {
      clearInterval(statsIntervalRef.current);
      statsIntervalRef.current = null;
    }

    console.log('⏹️  Parallel OCR tracking stopped');
  }, []);

  /**
   * Reset tracking session
   */
  const reset = useCallback(async () => {
    stop();
    sessionStartedRef.current = false;
    prevLevelRef.current = null;
    prevExpRef.current = null;
    setCurrentStats(null);

    try {
      await resetExpSession();
      ocrService.clearAllStores();
      console.log('🔄 Tracking session reset');
    } catch (err) {
      console.error('Reset failed:', err);
    }
  }, [stop]);

  /**
   * Effect: Monitor level and EXP changes to update ExpCalculator
   */
  useEffect(() => {
    if (!isTrackingRef.current) return;

    const unsubscribeLevel = useLevelStore.subscribe((state) => {
      const newLevel = state.level;
      if (newLevel === null || newLevel === prevLevelRef.current) return;

      prevLevelRef.current = newLevel;
      tryUpdateExpCalculator();
    });

    const unsubscribeExp = useExpStore.subscribe((state) => {
      const newExp = state.absolute;
      if (newExp === null || newExp === prevExpRef.current) return;

      prevExpRef.current = newExp;
      tryUpdateExpCalculator();
    });

    return () => {
      unsubscribeLevel();
      unsubscribeExp();
    };
  }, []);

  /**
   * Try to update ExpCalculator when both level and EXP are available
   */
  async function tryUpdateExpCalculator() {
    const level = useLevelStore.getState().level;
    const exp = useExpStore.getState().absolute;
    const percentage = useExpStore.getState().percentage;

    if (level === null || exp === null || percentage === null) {
      return; // Wait for both values
    }

    try {
      // Start session on first valid data
      if (!sessionStartedRef.current) {
        console.log('🟢 Starting EXP session:', { level, exp, percentage });
        await startExpSession(level, exp, percentage);
        sessionStartedRef.current = true;
      } else {
        // Update existing session
        console.log('🔵 Updating EXP data:', { level, exp, percentage });
        const stats = await addExpData(level, exp, percentage);
        setCurrentStats(stats);
      }
    } catch (err) {
      console.error('ExpCalculator update failed:', err);
    }
  }

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (isTrackingRef.current) {
        ocrService.stopAllLoops();
      }
      if (statsIntervalRef.current !== null) {
        clearInterval(statsIntervalRef.current);
      }
    };
  }, []);

  return {
    start,
    stop,
    reset,
    stats: currentStats,
    isRunning: () => ocrService.isRunning(),
  };
}
