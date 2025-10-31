import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Roi } from '../lib/tauri';
import { saveRoi, loadRoi, clearRoi } from '../lib/roiCommands';

export type RoiType = 'level' | 'exp' | 'inventory'; // | 'mapLocation' - commented out temporarily

interface RoiState {
  // Current ROI configurations
  levelRoi: Roi | null;
  expRoi: Roi | null;
  // mapLocationRoi: Roi | null; // Commented out temporarily

  // Loading states
  isLoading: boolean;
  error: string | null;

  // Actions
  setRoi: (type: RoiType, roi: Roi) => Promise<void>;
  getRoi: (type: RoiType) => Roi | null;
  removeRoi: (type: RoiType) => Promise<void>;
  loadAllRois: () => Promise<void>;
  clearError: () => void;
}

export const useRoiStore = create<RoiState>()(
  persist(
    (set, get) => ({
      // Initial state
      levelRoi: null,
      expRoi: null,
      // mapLocationRoi: null, // Commented out temporarily
      isLoading: false,
      error: null,

      // Set and persist ROI
      setRoi: async (type: RoiType, roi: Roi) => {
        set({ isLoading: true, error: null });
        try {
          // Save to backend
          await saveRoi(type, roi);

          // Update local state
          switch (type) {
            case 'level':
              set({ levelRoi: roi, isLoading: false });
              break;
            case 'exp':
              set({ expRoi: roi, isLoading: false });
              break;
            // case 'mapLocation': // Commented out temporarily
            //   set({ mapLocationRoi: roi, isLoading: false });
            //   break;
          }
        } catch (err) {
          const error = err instanceof Error ? err.message : 'Failed to save ROI';
          set({ error, isLoading: false });
          throw new Error(error);
        }
      },

      // Get ROI from state
      getRoi: (type: RoiType) => {
        const state = get();
        switch (type) {
          case 'level':
            return state.levelRoi;
          case 'exp':
            return state.expRoi;
          // case 'mapLocation': // Commented out temporarily
          //   return state.mapLocationRoi;
        }
      },

      // Remove ROI
      removeRoi: async (type: RoiType) => {
        set({ isLoading: true, error: null });
        try {
          // Clear from backend
          await clearRoi(type);

          // Update local state
          switch (type) {
            case 'level':
              set({ levelRoi: null, isLoading: false });
              break;
            case 'exp':
              set({ expRoi: null, isLoading: false });
              break;
            // case 'mapLocation': // Commented out temporarily
            //   set({ mapLocationRoi: null, isLoading: false });
            //   break;
          }
        } catch (err) {
          const error = err instanceof Error ? err.message : 'Failed to remove ROI';
          set({ error, isLoading: false });
          throw new Error(error);
        }
      },

      // Load all ROIs from backend
      loadAllRois: async () => {
        set({ isLoading: true, error: null });
        try {
          const [levelRoi, expRoi] = await Promise.all([
            loadRoi('level'),
            loadRoi('exp'),
            // loadRoi('mapLocation'), // Commented out temporarily
          ]);

          set({
            levelRoi,
            expRoi,
            // mapLocationRoi, // Commented out temporarily
            isLoading: false,
          });
        } catch (err) {
          const error = err instanceof Error ? err.message : 'Failed to load ROIs';
          set({ error, isLoading: false });
          throw new Error(error);
        }
      },

      // Clear error message
      clearError: () => set({ error: null }),
    }),
    {
      name: 'exp-tracker-roi-store', // localStorage key
      // Only persist the ROI data, not loading/error states
      partialize: (state) => ({
        levelRoi: state.levelRoi,
        expRoi: state.expRoi,
        // mapLocationRoi: state.mapLocationRoi, // Commented out temporarily
      }),
    }
  )
);
