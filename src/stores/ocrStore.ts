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
 * HP OCR Store - Independent state for HP recognition
 */
interface HpStore {
  hp: number | null;
  lastUpdated: number;
  error: string | null;
  setHp: (hp: number) => void;
  setError: (error: string) => void;
  clear: () => void;
}

export const useHpStore = create<HpStore>((set) => ({
  hp: null,
  lastUpdated: 0,
  error: null,
  setHp: (hp) => set({
    hp,
    lastUpdated: Date.now(),
    error: null,
  }),
  setError: (error) => set({ error }),
  clear: () => set({
    hp: null,
    lastUpdated: 0,
    error: null,
  }),
}));

/**
 * MP OCR Store - Independent state for MP recognition
 */
interface MpStore {
  mp: number | null;
  lastUpdated: number;
  error: string | null;
  setMp: (mp: number) => void;
  setError: (error: string) => void;
  clear: () => void;
}

export const useMpStore = create<MpStore>((set) => ({
  mp: null,
  lastUpdated: 0,
  error: null,
  setMp: (mp) => set({
    mp,
    lastUpdated: Date.now(),
    error: null,
  }),
  setError: (error) => set({ error }),
  clear: () => set({
    mp: null,
    lastUpdated: 0,
    error: null,
  }),
}));
