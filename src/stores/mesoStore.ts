import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface MesoState {
  startMeso: number | null;
  endMeso: number | null;
  hpPotionPrice: number;
  mpPotionPrice: number;

  setStartMeso: (meso: number | null) => void;
  setEndMeso: (meso: number | null) => void;
  setHpPotionPrice: (price: number) => void;
  setMpPotionPrice: (price: number) => void;
  resetSession: () => void;

  calculateMesoGained: () => number;
  calculatePotionCost: (hpUsed: number, mpUsed: number) => number;
  calculateNetProfit: (hpUsed: number, mpUsed: number) => number;
}

export const useMesoStore = create<MesoState>()(
  persist(
    (set, get) => ({
      startMeso: null,
      endMeso: null,
      hpPotionPrice: 0,
      mpPotionPrice: 0,

      setStartMeso: (meso: number | null) => set({ startMeso: meso }),
      
      setEndMeso: (meso: number | null) => set({ endMeso: meso }),
      
      setHpPotionPrice: (price: number) => set({ hpPotionPrice: Math.max(0, price) }),
      
      setMpPotionPrice: (price: number) => set({ mpPotionPrice: Math.max(0, price) }),
      
      resetSession: () => set({ startMeso: null, endMeso: null }),

      calculateMesoGained: () => {
        const { startMeso, endMeso } = get();
        if (startMeso === null || endMeso === null) return 0;
        return endMeso - startMeso;
      },

      calculatePotionCost: (hpUsed: number, mpUsed: number) => {
        const { hpPotionPrice, mpPotionPrice } = get();
        return (hpUsed * hpPotionPrice) + (mpUsed * mpPotionPrice);
      },

      calculateNetProfit: (hpUsed: number, mpUsed: number) => {
        const mesoGained = get().calculateMesoGained();
        const potionCost = get().calculatePotionCost(hpUsed, mpUsed);
        return mesoGained - potionCost;
      },
    }),
    {
      name: 'exp-tracker-meso-store',
    }
  )
);

