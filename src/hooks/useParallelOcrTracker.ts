import { useEffect, useRef, useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useRoiStore } from '../stores/roiStore';
import { useLevelStore, useExpStore, useHpPotionStore, useMpPotionStore } from '../stores/ocrStore';
import { startExpSession, addExpData, resetExpSession, type ExpStats } from '../lib/expCommands';

interface TrackingStats {
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
 * Hook for parallel OCR tracking with independent loops
 *
 * Features:
 * - 3 parallel tasks: Level, EXP, Inventory (with automatic ROI detection)
 * - Each OCR completes → waits 500ms → repeats
 * - Failures in one OCR don't affect others
 * - Inventory auto-detects region from full screen
 * - HP/MP potions counted from inventory via slot configuration
 * - Integrates with ExpCalculator backend
 */
export function useParallelOcrTracker() {
  const { levelRoi, expRoi } = useRoiStore();

  const isTrackingRef = useRef(false);
  const sessionStartedRef = useRef(false);
  const statsIntervalRef = useRef<number | null>(null);
  const [currentStats, setCurrentStats] = useState<ExpStats | null>(null);

  // Subscribe to level and EXP changes to update ExpCalculator
  const prevLevelRef = useRef<number | null>(null);
  const prevExpRef = useRef<number | null>(null);

  // Event unlisteners
  const unlistenersRef = useRef<UnlistenFn[]>([]);

  /**
   * Start OCR tracking via Rust backend
   */
  const start = useCallback(async () => {
    if (!levelRoi || !expRoi) {
      console.error('Level and EXP ROIs must be configured');
      return;
    }

    isTrackingRef.current = true;
    sessionStartedRef.current = false;

    // Clear previous refs
    prevLevelRef.current = null;
    prevExpRef.current = null;
    setCurrentStats(null);

    console.log('🚀 Starting Rust OCR tracker...');

    try {
      // Set up event listeners for real-time updates
      const levelUnlisten = await listen<{ level: number }>('ocr:level-update', async (event) => {
        console.log('⚡ [Event] Level update:', event.payload.level);
        useLevelStore.getState().setLevel({
          level: event.payload.level,
          raw_text: "",
        });

        // Refresh stats to update UI display
        try {
          const updatedStats = await invoke<ExpStats>('get_tracking_stats');
          setCurrentStats(updatedStats);
        } catch (err) {
          console.error('Failed to get updated stats after level change:', err);
        }
      });

      const expUnlisten = await listen<{ exp: number; percentage: number }>('ocr:exp-update', async (event) => {
        console.log('⚡ [Event] EXP update:', event.payload.exp, event.payload.percentage);
        useExpStore.getState().setExp({
          absolute: event.payload.exp,
          percentage: event.payload.percentage,
          raw_text: "",
        });

        // Update ExpCalculator if changed
        const level = useLevelStore.getState().level;
        if (level && (event.payload.exp !== prevExpRef.current || level !== prevLevelRef.current)) {
          prevLevelRef.current = level;
          prevExpRef.current = event.payload.exp;

          if (!sessionStartedRef.current) {
            await startExpSession(level, event.payload.exp, event.payload.percentage);
            sessionStartedRef.current = true;
          }
          await addExpData(level, event.payload.exp, event.payload.percentage);

          // Get complete stats including HP/MP from their independent calculators
          try {
            const updatedStats = await invoke<ExpStats>('get_tracking_stats');
            setCurrentStats(updatedStats);
          } catch (err) {
            console.error('Failed to get updated stats after EXP change:', err);
          }
        }
      });

      const hpPotionUnlisten = await listen<{ hp_potion_count: number }>('ocr:hp-potion-update', async (event) => {
        console.log('⚡ [Event] HP Potion update:', event.payload.hp_potion_count);
        useHpPotionStore.getState().setHpPotionCount(event.payload.hp_potion_count);

        // Update stats to reflect potion usage changes
        try {
          const updatedStats = await invoke<ExpStats>('get_tracking_stats');
          setCurrentStats(updatedStats);
        } catch (err) {
          console.error('Failed to get updated stats after HP potion change:', err);
        }
      });

      const mpPotionUnlisten = await listen<{ mp_potion_count: number }>('ocr:mp-potion-update', async (event) => {
        console.log('⚡ [Event] MP Potion update:', event.payload.mp_potion_count);
        useMpPotionStore.getState().setMpPotionCount(event.payload.mp_potion_count);

        // Update stats to reflect potion usage changes
        try {
          const updatedStats = await invoke<ExpStats>('get_tracking_stats');
          setCurrentStats(updatedStats);
        } catch (err) {
          console.error('Failed to get updated stats after MP potion change:', err);
        }
      });

      unlistenersRef.current = [levelUnlisten, expUnlisten, hpPotionUnlisten, mpPotionUnlisten];

      // Call Rust backend to start tracking (inventory auto-detects ROI)
      await invoke('start_ocr_tracking', {
        levelRoi,
        expRoi,
      });
      console.log('✅ Rust OCR tracker started with event-driven updates');
    } catch (error) {
      console.error('❌ Failed to start Rust OCR tracker:', error);
      isTrackingRef.current = false;
    }
  }, [levelRoi, expRoi]);

  /**
   * Stop OCR tracking via Rust backend
   */
  const stop = useCallback(async () => {
    isTrackingRef.current = false;

    // Clean up event listeners
    unlistenersRef.current.forEach(unlisten => unlisten());
    unlistenersRef.current = [];

    if (statsIntervalRef.current !== null) {
      clearInterval(statsIntervalRef.current);
      statsIntervalRef.current = null;
    }

    try {
      await invoke('stop_ocr_tracking');
      console.log('⏹️  Rust OCR tracker stopped');
    } catch (error) {
      console.error('❌ Failed to stop Rust OCR tracker:', error);
    }
  }, []);

  /**
   * Reset tracking session via Rust backend
   */
  const reset = useCallback(async () => {
    await stop();
    sessionStartedRef.current = false;
    prevLevelRef.current = null;
    prevExpRef.current = null;
    setCurrentStats(null);

    try {
      await invoke('reset_tracking');
      await resetExpSession();
      console.log('🔄 Rust tracking session reset');
    } catch (err) {
      console.error('❌ Reset failed:', err);
    }
  }, [stop]);


  // Cleanup on unmount
  useEffect(() => {
    return () => {
      // Clean up event listeners
      unlistenersRef.current.forEach(unlisten => unlisten());

      if (isTrackingRef.current) {
        invoke('stop_ocr_tracking').catch(console.error);
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
    isRunning: () => isTrackingRef.current,
  };
}
