import { useState, useEffect, useCallback, useRef } from 'react';
import { useRoiStore } from '../stores/roiStore';
import {
  startOcrTracking,
  stopOcrTracking,
  getTrackingStats,
  resetTracking,
  type TrackingStats,
} from '../lib/trackingCommands';

export interface ExpTrackerState {
  /** Current statistics */
  stats: TrackingStats | null;
  /** Whether tracking is active */
  isTracking: boolean;
  /** Last error message */
  error: string | null;
}

export interface UseExpTrackerReturn {
  state: ExpTrackerState;
  /** Start EXP tracking session */
  start: () => Promise<void>;
  /** Stop EXP tracking */
  stop: () => void;
  /** Reset EXP tracking session */
  reset: () => Promise<void>;
}

/**
 * Simplified hook for EXP tracking with Rust-managed state
 *
 * - All OCR processing happens in Rust (3 parallel tokio tasks: Level, EXP, Inventory)
 * - Inventory uses automatic ROI detection from full screen
 * - Frontend polls stats every 500ms
 * - No complex state management in frontend
 */
export function useExpTracker(): UseExpTrackerReturn {
  const { levelRoi, expRoi } = useRoiStore();

  const [state, setState] = useState<ExpTrackerState>({
    stats: null,
    isTracking: false,
    error: null,
  });

  const isTrackingRef = useRef(false);
  const pollingIntervalRef = useRef<number | null>(null);

  /**
   * Poll tracking stats from Rust backend
   */
  const pollStats = useCallback(async () => {
    if (!isTrackingRef.current) return;

    try {
      const stats = await getTrackingStats();
      setState(prev => ({
        ...prev,
        stats,
        error: stats.error,
      }));
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.error('❌ Failed to get tracking stats:', errorMsg);
      setState(prev => ({ ...prev, error: errorMsg }));
    }
  }, []);

  /**
   * Start tracking and polling
   */
  const start = useCallback(async () => {
    if (!levelRoi || !expRoi) {
      setState(prev => ({ ...prev, error: 'Level and EXP ROIs must be configured' }));
      return;
    }

    try {
      // Start Rust-side OCR tracking (inventory auto-detects ROI)
      await startOcrTracking(levelRoi, expRoi);

      isTrackingRef.current = true;
      setState(prev => ({ ...prev, isTracking: true, error: null }));

      // Start polling stats every 500ms
      pollingIntervalRef.current = window.setInterval(pollStats, 500);

      console.log('✅ Tracking started with Rust backend (auto inventory ROI)');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.error('❌ Failed to start tracking:', errorMsg);
      setState(prev => ({ ...prev, error: errorMsg }));
    }
  }, [levelRoi, expRoi, pollStats]);

  /**
   * Stop tracking and polling
   */
  const stop = useCallback(async () => {
    isTrackingRef.current = false;

    // Stop polling
    if (pollingIntervalRef.current !== null) {
      window.clearInterval(pollingIntervalRef.current);
      pollingIntervalRef.current = null;
    }

    try {
      await stopOcrTracking();
      setState(prev => ({ ...prev, isTracking: false }));
      console.log('⏹️  Tracking stopped');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.error('❌ Failed to stop tracking:', errorMsg);
    }
  }, []);

  /**
   * Reset tracking session
   */
  const reset = useCallback(async () => {
    await stop();

    try {
      await resetTracking();
      setState({
        stats: null,
        isTracking: false,
        error: null,
      });
      console.log('🔄 Tracking session reset');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.error('❌ Failed to reset tracking:', errorMsg);
      setState(prev => ({ ...prev, error: errorMsg }));
    }
  }, [stop]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (pollingIntervalRef.current !== null) {
        window.clearInterval(pollingIntervalRef.current);
      }
      if (isTrackingRef.current) {
        stopOcrTracking().catch(console.error);
      }
    };
  }, []);

  return {
    state,
    start,
    stop,
    reset,
  };
}
