import { create } from 'zustand';
import type { LevelResult, ExpResult } from '../lib/ocrCommands';

/**
 * Level OCR Store - Independent state for level recognition
 */
interface LevelStore {
  level: number | null;
  rawText: string | null;
  lastUpdated: number;
  error: string | null;
  setLevel: (data: LevelResult) => void;
  setError: (error: string) => void;
  clear: () => void;
}

export const useLevelStore = create<LevelStore>((set) => ({
  level: null,
  rawText: null,
  lastUpdated: 0,
  error: null,
  setLevel: (data) => set({
    level: data.level,
    rawText: data.raw_text,
    lastUpdated: Date.now(),
    error: null,
  }),
  setError: (error) => set({ error }),
  clear: () => set({
    level: null,
    rawText: null,
    lastUpdated: 0,
    error: null,
  }),
}));

/**
 * EXP OCR Store - Independent state for EXP recognition
 */
interface ExpStore {
  absolute: number | null;
  percentage: number | null;
  rawText: string | null;
  lastUpdated: number;
  error: string | null;
  setExp: (data: ExpResult) => void;
  setError: (error: string) => void;
  clear: () => void;
}

export const useExpStore = create<ExpStore>((set) => ({
  absolute: null,
  percentage: null,
  rawText: null,
  lastUpdated: 0,
  error: null,
  setExp: (data) => set({
    absolute: data.absolute,
    percentage: data.percentage,
    rawText: data.raw_text,
    lastUpdated: Date.now(),
    error: null,
  }),
  setError: (error) => set({ error }),
  clear: () => set({
    absolute: null,
    percentage: null,
    rawText: null,
    lastUpdated: 0,
    error: null,
  }),
}));

/**
 * HP Potion OCR Store - Independent state for HP potion count recognition
 */
interface HpPotionStore {
  hpPotionCount: number | null;
  lastUpdated: number;
  error: string | null;
  setHpPotionCount: (count: number) => void;
  setError: (error: string) => void;
  clear: () => void;
}

export const useHpPotionStore = create<HpPotionStore>((set) => ({
  hpPotionCount: null,
  lastUpdated: 0,
  error: null,
  setHpPotionCount: (count) => set({
    hpPotionCount: count,
    lastUpdated: Date.now(),
    error: null,
  }),
  setError: (error) => set({ error }),
  clear: () => set({
    hpPotionCount: null,
    lastUpdated: 0,
    error: null,
  }),
}));

/**
 * MP Potion OCR Store - Independent state for MP potion count recognition
 */
interface MpPotionStore {
  mpPotionCount: number | null;
  lastUpdated: number;
  error: string | null;
  setMpPotionCount: (count: number) => void;
  setError: (error: string) => void;
  clear: () => void;
}

export const useMpPotionStore = create<MpPotionStore>((set) => ({
  mpPotionCount: null,
  lastUpdated: 0,
  error: null,
  setMpPotionCount: (count) => set({
    mpPotionCount: count,
    lastUpdated: Date.now(),
    error: null,
  }),
  setError: (error) => set({ error }),
  clear: () => set({
    mpPotionCount: null,
    lastUpdated: 0,
    error: null,
  }),
}));
