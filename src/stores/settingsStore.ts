import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface SettingsState {
  // Opacity settings (0.0 - 1.0)
  backgroundOpacity: number;
  
  // Target duration in minutes (0 = disabled)
  targetDuration: number;

  // Actions
  setBackgroundOpacity: (opacity: number) => void;
  setTargetDuration: (minutes: number) => void;
  resetSettings: () => void;
}

const DEFAULT_SETTINGS = {
  backgroundOpacity: 0.95,
  targetDuration: 0, // Disabled by default
};

const MIN_OPACITY = 0.3; // Minimum 30% to prevent invisible UI
const MAX_OPACITY = 1.0; // Maximum 100%

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      ...DEFAULT_SETTINGS,

      setBackgroundOpacity: (opacity: number) =>
        set({ backgroundOpacity: Math.max(MIN_OPACITY, Math.min(MAX_OPACITY, opacity)) }),

      setTargetDuration: (minutes: number) =>
        set({ targetDuration: Math.max(0, minutes) }),

      resetSettings: () => set(DEFAULT_SETTINGS),
    }),
    {
      name: 'exp-tracker-settings', // localStorage key
    }
  )
);
