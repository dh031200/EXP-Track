### 프론트엔드 라이브러리(Invoke 래퍼 및 유틸)

각 파일은 Rust Tauri 명령어를 안전하게 호출하기 위한 래퍼 함수와 유틸을 제공합니다.

---

### `src/lib/tauri.ts`
- **Roi**: `{ x:number, y:number, width:number, height:number }`
- **initScreenCapture(): Promise<void>** — 화면 캡처 초기화
- **getScreenDimensions(): Promise<[number, number]>** — 화면 크기
- **captureRegion(roi: Roi): Promise<number[]>** — ROI 캡처 PNG 바이트
- **captureFullScreen(): Promise<number[]>** — 전체 화면 PNG 바이트
- **bytesToDataUrl(bytes: number[]): string** — PNG 바이트 → `data:image/png;base64,...`
- **maximizeWindowForROI(): Promise<{width;height;x;y}>** — ROI 선택용 창 리사이즈, 기존 상태 반환
- **restoreWindow(state): Promise<void>** — 창 상태 복원
- **setAlwaysOnTop(flag: boolean): Promise<void>** — 항상 위 설정

예시:
```ts
import { initScreenCapture, captureRegion, bytesToDataUrl } from '@/src/lib/tauri';
await initScreenCapture();
const pngBytes = await captureRegion({ x: 100, y: 100, width: 240, height: 60 });
const dataUrl = bytesToDataUrl(pngBytes);
```

---

### `src/lib/ocrCommands.ts`
- 타입: `LevelResult { level, raw_text }`, `ExpResult { absolute, percentage, raw_text }`, `MapResult { map_name, raw_text }`
- **recognizeLevel(base64): Promise<LevelResult>**
- **recognizeExp(base64): Promise<ExpResult>**
- **recognizeMap(base64): Promise<MapResult>** — 현재 Rust에서 미구현
- **recognizeHpPotionCount(base64): Promise<number>**
- **recognizeMpPotionCount(base64): Promise<number>**
- **checkOcrHealth(): Promise<boolean>** — Python 서버 상태
- **dataUrlToBase64(dataUrl): string** — prefix 제거
- **blobToBase64(blob): Promise<string>** — Blob → base64

예시:
```ts
import { recognizeExp, blobToBase64 } from '@/src/lib/ocrCommands';
const b64 = await blobToBase64(imageBlob);
const exp = await recognizeExp(b64);
```

---

### `src/lib/expCommands.ts`
- **ExpStats**: 누적 경험치/퍼센트/시간/포션 사용량 포함 통계 타입
- **startExpSession(level, exp, pct, meso?) → Promise<string>**
- **addExpData(level, exp, pct, meso?) → Promise<ExpStats>**
- **resetExpSession() → Promise<string>**
- 포맷터: `formatElapsedTime(sec)`, `formatNumber(n)`, `formatPercentage(p)`, `formatCompact(n)`

예시:
```ts
import { startExpSession, addExpData } from '@/src/lib/expCommands';
await startExpSession(126, 5_509_611, 12.34);
const stats = await addExpData(126, 5_520_000, 12.50);
```

---

### `src/lib/trackingCommands.ts`
- **TrackingStats**: Rust 추적기에서 제공하는 현재 통계 타입
- **startOcrTracking(levelRoi, expRoi, hpRoi, mpRoi) → Promise<void>**
- **stopOcrTracking() → Promise<void>**
- **getTrackingStats() → Promise<TrackingStats>**
- **resetTracking() → Promise<void>**

예시:
```ts
import { startOcrTracking, getTrackingStats } from '@/src/lib/trackingCommands';
await startOcrTracking(level, exp, hp, mp);
const stats = await getTrackingStats();
```

---

### `src/lib/roiCommands.ts`
- **RoiType**: `'level' | 'exp' | 'hp' | 'mp' | 'meso'` (주의: Rust는 현재 `meso` 비활성)
- **saveRoi(type, roi) → Promise<void>**
- **loadRoi(type) → Promise<Roi|null>**
- **getAllRois() → Promise<{ level, exp, meso }>`** (Rust 측은 hp/mp 포함 JSON 반환)
- **clearRoi(type) → Promise<void>**
- **getConfigPath() → Promise<string>**

예시:
```ts
import { saveRoi, loadRoi } from '@/src/lib/roiCommands';
await saveRoi('level', { x: 120, y: 80, width: 180, height: 48 });
const levelRoi = await loadRoi('level');
```
