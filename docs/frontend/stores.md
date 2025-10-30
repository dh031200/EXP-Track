### Zustand 스토어

각 스토어는 독립적인 책임을 가지며 일부는 `persist`로 로컬 저장소에 보존됩니다.

---

### `useRoiStore`
상태:
- `levelRoi | expRoi | hpRoi | mpRoi: Roi|null`
- `isLoading: boolean`, `error: string|null`

액션:
- `setRoi(type, roi): Promise<void>` — 저장 + 상태 반영
- `getRoi(type): Roi|null`
- `removeRoi(type): Promise<void>`
- `loadAllRois(): Promise<void>`
- `clearError(): void`

Persist 키: `exp-tracker-roi-store` (ROI만 부분 보존)

예시:
```ts
const { setRoi, loadAllRois } = useRoiStore();
await loadAllRois();
await setRoi('level', { x: 100, y: 80, width: 200, height: 40 });
```

---

### `useLevelStore` / `useExpStore` / `useHpPotionStore` / `useMpPotionStore`
목적: OCR 결과 각각의 독립 상태

공통:
- 최근 업데이트 시각/에러 및 setter 제공
- `clear()`로 초기화 가능

예시(Exp):
```ts
const { absolute, percentage, setExp } = useExpStore();
setExp({ absolute: 5509611, percentage: 12.34, raw_text: '...' });
```

---

### `useSettingsStore`
상태:
- `backgroundOpacity: number (0.3~1.0)`
- `targetDuration: number (분, 0=비활성)`

액션:
- `setBackgroundOpacity(opacity)` — 경계 자동 클램프
- `setTargetDuration(minutes)` — 0 이상
- `resetSettings()`

Persist 키: `exp-tracker-settings`

---

### `useTimerSettingsStore`
타입:
- `AverageInterval = 'none'|'1min'|'5min'|'10min'|'30min'|'1hour'`
- `AutoStopInterval = 'none'|'5min'|'15min'|'30min'|'1hour'`
- `AverageCalculationMode = 'prediction'|'per_interval'`

상태:
- `selectedAverageInterval`, `autoStopInterval`, `averageCalculationMode`
- `showTotalTime: boolean`, `showSessionCount: boolean`

액션:
- `setAverageInterval`, `setAutoStopInterval`, `setAverageCalculationMode`
- `toggleTotalTime`, `toggleSessionCount`, `resetToDefaults`

Persist 키: `exp-tracker-timer-settings`

---

### `useTrackingStore`
상태:
- `state: 'idle'|'tracking'|'paused'`
- `elapsedSeconds`, `pausedSeconds`, `sessionStartTime`, `lastPauseTime`

액션/계산:
- `startTracking()`, `pauseTracking()`, `resetTracking()`
- `incrementTimer()`
- `getActiveDuration() → number` (경과-일시정지 합)

---

### `useSessionStore`
상태:
- `sessions: Session[]`, `currentSession: Session|null`

액션:
- `startSession()`, `endSession()`
- `updateSessionDuration(duration, pausedDuration)`
- `deleteSession(id)`, `clearAllSessions()`

게터:
- `getTotalSessions()`, `getTotalTrackingTime()`, `getAverageDuration()`
- `getRecentSessions(count)`

Persist 키: `exp-tracker-session-store` (히스토리만 보존)

---

### `useMesoStore`
상태:
- `startMeso|null`, `endMeso|null`, `hpPotionPrice`, `mpPotionPrice`

액션/계산:
- `setStartMeso`, `setEndMeso`, `setHpPotionPrice`, `setMpPotionPrice`, `resetSession`
- `calculateMesoGained()`, `calculatePotionCost(hpUsed, mpUsed)`, `calculateNetProfit(hpUsed, mpUsed)`

Persist 키: `exp-tracker-meso-store`
