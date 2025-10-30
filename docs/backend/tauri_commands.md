### Tauri 백엔드 명령어(API)

이 문서는 프론트엔드에서 `@tauri-apps/api`를 통해 호출되는 Rust 측 공개 명령어를 정리합니다. 각 명령어는 TypeScript 래퍼가 제공되므로 가급적 래퍼를 통해 사용하세요.

- 기본 호출: `invoke('<command_name>', { ...args })`
- 권장: `src/lib/*.ts` 래퍼 함수 사용

---

### 화면 캡처(Screen Capture)
- **init_screen_capture() → void**: 화면 캡처 초기화. 첫 호출 필요.
- **get_screen_dimensions() → [number, number]**: 논리 좌표 기준 화면 크기.
- **capture_region(roi: {x,y,width,height}) → Uint8 PNG bytes**: 지정 영역 캡처.
- **capture_full_screen() → Uint8 PNG bytes**: 전체 화면 캡처.

예시 (권장 래퍼 사용):
```ts
import { initScreenCapture, getScreenDimensions, captureRegion } from '@/src/lib/tauri';

await initScreenCapture();
const [w, h] = await getScreenDimensions();
const bytes = await captureRegion({ x: 100, y: 100, width: 300, height: 80 });
```

---

### 설정/ROI(Config & ROI)
- **save_roi(roiType, roi) → void**: ROI 저장. roiType: `level|exp|hp|mp`.
- **load_roi(roiType) → Roi|null**: 해당 ROI 로드.
- **get_all_rois() → { level, exp, hp, mp }**: 전체 ROI 조회.
- **clear_roi(roiType) → void**: 해당 ROI 제거.
- **save_config(config) → void / load_config() → AppConfig**: 전체 설정 저장/로드.
- **get_config_path() → string**: 설정 파일 경로.
- **save_roi_preview(roiType, imageDataBase64) → string**: 임시 폴더에 미리보기 PNG 저장, 경로 반환.
- **get_roi_preview(roiType) → data:image/png;base64,...**: 미리보기 이미지 로드.
- **open_roi_preview(roiType) → void**: 시스템 뷰어로 미리보기 열기.

주의: `meso` ROI는 현재 Rust 측에서 비활성화 상태입니다.

예시:
```ts
import { saveRoi, loadRoi, getConfigPath } from '@/src/lib/roiCommands';

await saveRoi('exp', { x: 300, y: 100, width: 260, height: 50 });
const exp = await loadRoi('exp');
const path = await getConfigPath();
```

---

### OCR(이미지 인식)
- **recognize_level(image_base64) → { level, raw_text }**
- **recognize_exp(image_base64) → { absolute, percentage, raw_text }**
- **recognize_map(image_base64) → MapResult**: 현재 미구현(에러 문자열 반환)
- **recognize_hp_potion_count(image_base64) → number**
- **recognize_mp_potion_count(image_base64) → number**
- **recognize_all_parallel(level_b64, exp_b64, hp_b64, mp_b64) → { level?, exp?, hp?, mp? }**: 병렬 수행
- **check_ocr_health() → boolean**: Python OCR 서버 상태 점검

예시 (권장 래퍼 사용):
```ts
import { recognizeLevel, recognizeExp, checkOcrHealth } from '@/src/lib/ocrCommands';

const ok = await checkOcrHealth();
const level = await recognizeLevel(base64);
const exp = await recognizeExp(base64);
```

---

### EXP 세션(통계 계산)
- **start_exp_session(level, exp, percentage, meso?) → string**: 세션 시작.
- **add_exp_data(level, exp, percentage, meso?) → ExpStats**: 스냅숏 추가 → 최신 통계 반환.
- **reset_exp_session() → string**: 세션 초기화.

예시 (권장 래퍼 사용):
```ts
import { startExpSession, addExpData, resetExpSession } from '@/src/lib/expCommands';

await startExpSession(126, 5509611, 12.34);
const stats = await addExpData(126, 5510000, 12.50);
await resetExpSession();
```

---

### 추적(토큰 관리형 병렬 OCR)
- **start_ocr_tracking(levelRoi, expRoi, hpRoi, mpRoi) → void**: Rust 측 4병렬 OCR 시작.
- **stop_ocr_tracking() → void**: 정지.
- **get_tracking_stats() → TrackingStats**: 현재 누적 통계.
- **reset_tracking() → void**: Rust 추적 상태 초기화.

예시 (권장 래퍼 사용):
```ts
import { startOcrTracking, getTrackingStats, stopOcrTracking } from '@/src/lib/trackingCommands';

await startOcrTracking(level, exp, hp, mp);
const stats = await getTrackingStats();
await stopOcrTracking();
```

---

### 인코딩 팁
- 이미지 바이트 → base64: 프론트엔드에서 `bytesToDataUrl` 또는 `blobToBase64` 활용
- `data:image/png;base64,...` 형태에서 순수 base64만 필요하면 `split(',')[1]`
