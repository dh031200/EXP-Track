import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface Session {
  id: string;
  startTime: number;
  endTime: number | null;
  duration: number; // Total seconds
  pausedDuration: number; // Total seconds paused

  // Experience data (placeholder for future OCR implementation)
  startLevel: number | null;
  endLevel: number | null;
  startExp: number | null;
  endExp: number | null;
  expGained: number | null;

  // Map location
  mapLocation: string | null;
}

interface SessionStore {
  sessions: Session[];
  currentSession: Session | null;

  // Actions
  startSession: () => void;
  endSession: () => void;
  updateSessionDuration: (duration: number, pausedDuration: number) => void;
  deleteSession: (id: string) => void;
  clearAllSessions: () => void;

  // Computed values
  getTotalSessions: () => number;
  getTotalTrackingTime: () => number;
  getAverageDuration: () => number;
  getRecentSessions: (count: number) => Session[];
}

export const useSessionStore = create<SessionStore>()(
  persist(
    (set, get) => ({
      sessions: [],
      currentSession: null,

      startSession: () => {
        const newSession: Session = {
          id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          startTime: Date.now(),
          endTime: null,
          duration: 0,
          pausedDuration: 0,
          startLevel: null,
          endLevel: null,
          startExp: null,
          endExp: null,
          expGained: null,
          mapLocation: null,
        };
        set({ currentSession: newSession });
      },

      endSession: () => {
        const { currentSession, sessions } = get();
        if (currentSession) {
          const completedSession: Session = {
            ...currentSession,
            endTime: Date.now(),
          };

          // Add to history and keep only last 100 sessions
          const updatedSessions = [completedSession, ...sessions].slice(0, 100);

          set({
            sessions: updatedSessions,
            currentSession: null,
          });
        }
      },

      updateSessionDuration: (duration: number, pausedDuration: number) => {
        const { currentSession } = get();
        if (currentSession) {
          set({
            currentSession: {
              ...currentSession,
              duration,
              pausedDuration,
            },
          });
        }
      },

      deleteSession: (id: string) => {
        set((state) => ({
          sessions: state.sessions.filter((s) => s.id !== id),
        }));
      },

      clearAllSessions: () => {
        set({ sessions: [] });
      },

      getTotalSessions: () => {
        return get().sessions.length;
      },

      getTotalTrackingTime: () => {
        return get().sessions.reduce((total, session) => total + session.duration, 0);
      },

      getAverageDuration: () => {
        const { sessions } = get();
        if (sessions.length === 0) return 0;
        const total = sessions.reduce((sum, s) => sum + s.duration, 0);
        return Math.floor(total / sessions.length);
      },

      getRecentSessions: (count: number) => {
        return get().sessions.slice(0, count);
      },
    }),
    {
      name: 'exp-tracker-session-store',
      partialize: (state) => ({
        sessions: state.sessions,
      }),
    }
  )
);
