import { create } from 'zustand';

export type TrackingState = 'idle' | 'tracking' | 'paused';

interface TrackingStore {
  state: TrackingState;
  elapsedSeconds: number;
  pausedSeconds: number;
  sessionStartTime: number | null;
  lastPauseTime: number | null;

  // Actions
  startTracking: () => void;
  pauseTracking: () => void;
  resetTracking: () => void;
  incrementTimer: () => void;
  getActiveDuration: () => number;
}

export const useTrackingStore = create<TrackingStore>((set, get) => ({
  state: 'idle',
  elapsedSeconds: 0,
  pausedSeconds: 0,
  sessionStartTime: null,
  lastPauseTime: null,

  startTracking: () =>
    set((state) => {
      const now = Date.now();
      let newPausedSeconds = state.pausedSeconds;

      // If resuming from pause, add the paused duration
      if (state.state === 'paused' && state.lastPauseTime) {
        const pauseDuration = Math.floor((now - state.lastPauseTime) / 1000);
        newPausedSeconds += pauseDuration;
      }

      return {
        state: 'tracking',
        sessionStartTime: state.sessionStartTime ?? now,
        lastPauseTime: null,
        pausedSeconds: newPausedSeconds,
      };
    }),

  pauseTracking: () =>
    set({
      state: 'paused',
      lastPauseTime: Date.now(),
    }),

  resetTracking: () =>
    set({
      state: 'idle',
      elapsedSeconds: 0,
      pausedSeconds: 0,
      sessionStartTime: null,
      lastPauseTime: null,
    }),

  incrementTimer: () =>
    set((state) => ({
      elapsedSeconds: state.elapsedSeconds + 1,
    })),

  getActiveDuration: () => {
    const state = get();
    return state.elapsedSeconds - state.pausedSeconds;
  },
}));
