import { useState, useEffect, useCallback, useRef } from 'react';
import { captureRegion } from '../lib/tauri';
import { recognizeLevel, recognizeExp, recognizeMap } from '../lib/ocrCommands';
import { startExpSession, addExpData, resetExpSession, type ExpStats } from '../lib/expCommands';
import { useRoiStore } from '../stores/roiStore';

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

export interface ExpTrackerState {
  /** Current statistics (null if session not started) */
  stats: ExpStats | null;
  /** Current character level */
  level: number | null;
  /** Current EXP absolute value */
  exp: number | null;
  /** Current percentage to next level */
  percentage: number | null;
  /** Current map location */
  mapName: string | null;
  /** Whether tracking is active */
  isTracking: boolean;
  /** Last error message */
  error: string | null;
  /** Whether currently polling OCR */
  isPolling: boolean;
  /** OCR status: 'success' | 'warning' | 'error' */
  ocrStatus: 'success' | 'warning' | 'error';
}

export interface UseExpTrackerReturn {
  state: ExpTrackerState;
  /** Start EXP tracking session */
  start: () => Promise<void>;
  /** Stop EXP tracking (pause polling) */
  stop: () => void;
  /** Reset EXP tracking session */
  reset: () => Promise<void>;
  /** Manually trigger OCR update */
  update: () => Promise<void>;
}

/**
 * Hook for EXP tracking with OCR polling
 *
 * Features:
 * - Automatic OCR polling every 1 second when tracking
 * - Integrates Level, EXP, and Map OCR recognition
 * - Manages ExpCalculator backend state
 * - Provides real-time statistics
 */
export function useExpTracker(): UseExpTrackerReturn {
  const { levelRoi, expRoi, mapLocationRoi } = useRoiStore();

  const [state, setState] = useState<ExpTrackerState>({
    stats: null,
    level: null,
    exp: null,
    percentage: null,
    mapName: null,
    isTracking: false,
    error: null,
    isPolling: false,
    ocrStatus: 'success',
  });

  const intervalRef = useRef<number | null>(null);
  const isTrackingRef = useRef(false);
  const isPollingRef = useRef(false);
  const sessionStartedRef = useRef(false);

  /**
   * Perform OCR on all configured ROIs and update calculator
   */
  const performOcrUpdate = useCallback(async () => {
    // Prevent multiple OCR operations from running simultaneously
    if (isPollingRef.current) {
      console.log('Skipping OCR - previous operation still in progress');
      return;
    }

    if (!levelRoi || !expRoi) {
      setState(prev => ({ ...prev, error: 'ROI not configured' }));
      return;
    }

    isPollingRef.current = true;
    setState(prev => ({ ...prev, isPolling: true, error: null }));

    try {
      // Capture and recognize level
      const levelBytes = await captureRegion(levelRoi);
      const levelBase64 = bytesToBase64(levelBytes);
      const levelResult = await recognizeLevel(levelBase64);
      console.log('🔵 OCR Level Result:', levelResult);

      // Capture and recognize EXP
      const expBytes = await captureRegion(expRoi);
      const expBase64 = bytesToBase64(expBytes);
      const expResult = await recognizeExp(expBase64);
      console.log('🔵 OCR EXP Result:', expResult);

      // Map OCR temporarily disabled for minimal UI
      // // Capture and recognize map (optional)
      // let mapName = state.mapName;
      // if (mapLocationRoi) {
      //   try {
      //     const mapBytes = await captureRegion(mapLocationRoi);
      //     const mapBase64 = bytesToBase64(mapBytes);
      //     const mapResult = await recognizeMap(mapBase64);
      //     mapName = mapResult.map_name;
      //   } catch (err) {
      //     console.warn('Map recognition failed:', err);
      //     // Don't fail the whole update if map recognition fails
      //   }
      // }

      // If this is the first update, start the session
      if (!sessionStartedRef.current) {
        console.log('🟢 Starting new EXP session with:', {
          level: levelResult.level,
          absolute: expResult.absolute,
          percentage: expResult.percentage
        });
        await startExpSession(
          levelResult.level,
          expResult.absolute,
          expResult.percentage
        );
        sessionStartedRef.current = true;
      }

      // Add new data and get updated stats
      console.log('🟢 Adding EXP data:', {
        level: levelResult.level,
        absolute: expResult.absolute,
        percentage: expResult.percentage
      });
      const stats = await addExpData(
        levelResult.level,
        expResult.absolute,
        expResult.percentage
      );
      console.log('🟢 Updated stats:', stats);

      setState(prev => ({
        ...prev,
        stats,
        level: levelResult.level,
        exp: expResult.absolute,
        percentage: expResult.percentage,
        // mapName, // Map OCR disabled
        isPolling: false,
        error: null,
        ocrStatus: 'success',
      }));
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.warn('OCR update failed:', errorMsg);

      // Determine OCR status based on error
      const isWarning = errorMsg.includes('out of valid range') ||
                       errorMsg.includes('Could not parse');

      setState(prev => ({
        ...prev,
        isPolling: false,
        error: errorMsg, // Store error but don't display it
        ocrStatus: isWarning ? 'warning' : 'error',
      }));
    } finally {
      // Always reset polling flag when done
      isPollingRef.current = false;
    }
  }, [levelRoi, expRoi, mapLocationRoi]);

  /**
   * Start tracking
   */
  const start = useCallback(async () => {
    if (!levelRoi || !expRoi) {
      setState(prev => ({ ...prev, error: 'ROI not configured' }));
      return;
    }

    isTrackingRef.current = true;
    setState(prev => ({ ...prev, isTracking: true, error: null }));

    // Perform initial OCR update
    await performOcrUpdate();

    // Start polling interval (1 second)
    if (intervalRef.current !== null) {
      clearInterval(intervalRef.current);
    }

    intervalRef.current = window.setInterval(() => {
      if (isTrackingRef.current) {
        performOcrUpdate();
      }
    }, 1000);
  }, [levelRoi, expRoi, performOcrUpdate]);

  /**
   * Stop tracking (pause polling)
   */
  const stop = useCallback(() => {
    isTrackingRef.current = false;
    setState(prev => ({ ...prev, isTracking: false }));

    if (intervalRef.current !== null) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  /**
   * Reset tracking session
   */
  const reset = useCallback(async () => {
    stop();
    sessionStartedRef.current = false;

    try {
      await resetExpSession();
      setState({
        stats: null,
        level: null,
        exp: null,
        percentage: null,
        mapName: null,
        isTracking: false,
        error: null,
        isPolling: false,
        ocrStatus: 'success',
      });
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      console.error('Reset failed:', errorMsg);
      setState(prev => ({ ...prev, error: errorMsg }));
    }
  }, [stop]);

  /**
   * Manually trigger OCR update
   */
  const update = useCallback(async () => {
    if (!isTrackingRef.current) {
      return;
    }
    await performOcrUpdate();
  }, [performOcrUpdate]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (intervalRef.current !== null) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  return {
    state,
    start,
    stop,
    reset,
    update,
  };
}
