### React 컴포넌트

주요 UI 컴포넌트의 목적과 공개 props를 정리합니다. 모든 컴포넌트는 TypeScript로 작성되었습니다.

---

### `TitleBar`
- 목적: 커스텀 타이틀바(드래그/최소화/종료)
- Props: 없음

사용 예:
```tsx
import { TitleBar } from '@/src/components/TitleBar';

export default function Layout() {
  return <TitleBar />;
}
```

---

### `Settings`
- 목적: 배경 투명도, 타이머 목표 시간 등 UI 설정
- Props: 없음

---

### `ExpTrackerDisplay`
- 목적: 누적/평균 경험치, 포션 사용량 등 집계 카드 표시
- Props:
  - `stats: ExpStats | null`
  - `isTracking: boolean`
  - `error: string | null`
  - `averageData: { label: string; value: string } | null`
  - `calculationMode: 'prediction' | 'per_interval'`
  - `intervalLabel: string`
  - `potionUsage: { hpPerMinute: number; mpPerMinute: number }`

사용 예:
```tsx
<ExpTrackerDisplay
  stats={stats}
  isTracking={isTracking}
  error={error}
  averageData={{ label: '5분', value: '12,345' }}
  calculationMode="per_interval"
  intervalLabel="5분"
  potionUsage={{ hpPerMinute: 1.2, mpPerMinute: 0.8 }}
/>
```

---

### `RoiManager`
- 목적: ROI(레벨/경험치/HP/MP) 설정 및 미리보기
- Props:
  - `onSelectingChange?: (isSelecting: boolean) => void`

---

### `CompactRoiManager`
- 목적: 모달 내부에서 사용하는 콤팩트 ROI 설정 UI
- Props:
  - `onSelectingChange?: (isSelecting: boolean) => void`

---

### `RoiSelector`
- 목적: 전체 화면 오버레이에서 드래그로 ROI 선택
- Props:
  - `onRoiSelected: (roi: {x,y,width,height}) => void`
  - `onCancel?: () => void`

사용 예:
```tsx
<RoiSelector onRoiSelected={(roi) => console.log(roi)} onCancel={() => {/* ... */}} />
```

---

### `RoiConfigModal`
- 목적: ROI 설정 모달(내부에서 `CompactRoiManager` 사용)
- Props:
  - `isOpen: boolean`
  - `onClose: () => void`
  - `onSelectingChange?: (isSelecting: boolean) => void`

---

### `TimerSettingsModal`
- 목적: 타이머 관련 옵션(평균 구간/자동 정지/표시 방식)
- Props:
  - `isOpen: boolean`
  - `onClose: () => void`

---

### `HistoryDashboard`
- 목적: 세션 기록 및 통계 대시보드(최근 세션, 총합, 평균 등)
- Props:
  - `isOpen: boolean`
  - `onClose: () => void`

---

### `RoiDemo`
- 목적: ROI 선택 및 캡처 데모(개발/테스트용)
- Props:
  - `onSelectingChange?: (isSelecting: boolean) => void`
