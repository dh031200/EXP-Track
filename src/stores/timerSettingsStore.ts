import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type AverageInterval = 'none' | '5min' | '10min' | '30min' | '1hour';
export type AutoStopInterval = 'none' | '5min' | '15min' | '30min' | '1hour';

interface TimerSettings {
  // Main screen average interval (single selection)
  selectedAverageInterval: AverageInterval;

  // Auto stop timer
  autoStopInterval: AutoStopInterval;

  // Display preferences
  showTotalTime: boolean;
  showSessionCount: boolean;
}

interface TimerSettingsStore extends TimerSettings {
  // Actions
  setAverageInterval: (interval: AverageInterval) => void;
  setAutoStopInterval: (interval: AutoStopInterval) => void;
  toggleTotalTime: () => void;
  toggleSessionCount: () => void;
  resetToDefaults: () => void;
}

const DEFAULT_SETTINGS: TimerSettings = {
  selectedAverageInterval: 'none',
  autoStopInterval: 'none',
  showTotalTime: true,
  showSessionCount: true,
};

export const useTimerSettingsStore = create<TimerSettingsStore>()(
  persist(
    (set) => ({
      ...DEFAULT_SETTINGS,

      setAverageInterval: (interval: AverageInterval) =>
        set({ selectedAverageInterval: interval }),

      setAutoStopInterval: (interval: AutoStopInterval) =>
        set({ autoStopInterval: interval }),

      toggleTotalTime: () =>
        set((state) => ({ showTotalTime: !state.showTotalTime })),

      toggleSessionCount: () =>
        set((state) => ({ showSessionCount: !state.showSessionCount })),

      resetToDefaults: () => set(DEFAULT_SETTINGS),
    }),
    {
      name: 'exp-tracker-timer-settings',
    }
  )
);
