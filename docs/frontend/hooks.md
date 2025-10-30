### 커스텀 훅(Hooks)

프론트엔드에서 추적 제어와 상태 구독을 단순화합니다.

---

### `useExpTracker()`
Rust 관리형 병렬 OCR(`start_ocr_tracking`)을 사용하고 500ms 주기로 통계를 폴링합니다.

반환값:
- `state: { stats: ExpStats|null; isTracking: boolean; error: string|null }`
- `start(): Promise<void>` — 4개 ROI가 모두 설정되어야 시작
- `stop(): void` — 폴링 중지 및 Rust 정지
- `reset(): Promise<void>` — 세션/상태 초기화

예시:
```tsx
import { useExpTracker } from '@/src/hooks/useExpTracker';

export function StartButton() {
  const { state, start, stop } = useExpTracker();
  return (
    <button onClick={state.isTracking ? stop : start}>
      {state.isTracking ? '정지' : '시작'}
    </button>
  );
}
```

---

### `useParallelOcrTracker()`
이벤트 기반 실시간 업데이트와 ExpCalculator 연동을 포함하는 고급 훅입니다.

반환값:
- `start(): Promise<void>` — Rust 추적 + 이벤트 리스너 등록
- `stop(): Promise<void>` — 추적 중지 및 리스너 해제
- `reset(): Promise<void>` — Rust 추적 및 EXP 세션 초기화
- `stats: ExpStats|null` — 최신 통계 스냅샷
- `isRunning(): boolean` — 현재 실행 여부

예시:
```tsx
import { useParallelOcrTracker } from '@/src/hooks/useParallelOcrTracker';

export function TrackerControls() {
  const { start, stop, reset, stats } = useParallelOcrTracker();
  return (
    <div>
      <button onClick={start}>시작</button>
      <button onClick={stop}>정지</button>
      <button onClick={reset}>리셋</button>
      <div>총 경험치: {stats?.total_exp ?? 0}</div>
    </div>
  );
}
```
