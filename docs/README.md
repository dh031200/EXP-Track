## 문서 개요 (EXP Tracker)

이 문서는 일반 사용자와 한국 개발자를 모두 대상으로, 앱의 공개 API/함수/컴포넌트 사용법과 예시를 한 곳에 정리합니다. 실사용자는 "시작 가이드"와 "컴포넌트"를, 개발자는 "프론트엔드/백엔드 API"를 우선 확인하세요.

### 목차
- 프론트엔드
  - [라이브러리(Invoke 래퍼/유틸)](./frontend/lib.md)
  - [커스텀 훅(Hooks)](./frontend/hooks.md)
  - [컴포넌트](./frontend/components.md)
  - [Zustand 스토어](./frontend/stores.md)
- 백엔드
  - [Tauri 명령어(API)](./backend/tauri_commands.md)
  - [모델(Models)](./backend/models.md)
  - [서비스 개요](./backend/services.md)
- Python OCR 서버
  - [REST API와 실행 방법](./python/ocr_server.md)

### 빠른 시작(사용자)
1) 설정 → 영역 설정에서 레벨/경험치/HP/MP 영역을 드래그로 지정합니다.
2) 추적 시작을 누르면 자동으로 OCR 및 통계가 갱신됩니다.
3) "히스토리"에서 이전 세션을 확인할 수 있습니다.

### 빠른 시작(개발자)
- 화면 캡처 초기화 → ROI 캡처 → base64 변환 → OCR 호출 흐름
```ts
import { initScreenCapture, captureRegion, bytesToDataUrl } from '@/src/lib/tauri';
import { recognizeLevel, dataUrlToBase64 } from '@/src/lib/ocrCommands';

await initScreenCapture();
const bytes = await captureRegion({ x:100, y:80, width:240, height:60 });
const b64 = dataUrlToBase64(bytesToDataUrl(bytes));
const level = await recognizeLevel(b64);
```
