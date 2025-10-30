### Python OCR 서버 (FastAPI + RapidOCR)

Rust 백엔드가 HTTP 클라이언트로 호출하는 OCR 서버입니다. 앱 실행 시 자동으로 백그라운드에서 구동되며, 직접 실행/테스트도 가능합니다.

- 프레임워크: FastAPI
- 엔진: RapidOCR (PaddleOCR 모델)
- 포트: 39835 (기본)
- 병렬 처리: 엔진 4개 풀 + ThreadPoolExecutor(4)

---

### 설치 및 실행
```bash
cd python_ocr_server
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -r requirements.txt
python main.py  # 또는: uvicorn main:app --host 127.0.0.1 --port 39835
```

앱 내부에선 Rust가 자동으로 서버를 구동/종료합니다.

---

### 엔드포인트

#### POST /ocr
- 요청: `{ "image_base64": "<PNG base64>" }`
- 응답: `OcrResponse { boxes: TextBox[], raw_text: string }`
- `TextBox { box: number[4][2], text: string, score: number }`

예시(curl):
```bash
BASE64=$(base64 -w0 sample.png)
curl -s -X POST http://127.0.0.1:39835/ocr \
  -H "Content-Type: application/json" \
  -d "{\"image_base64\":\"$BASE64\"}"
```

프론트엔드 팁:
```ts
import { dataUrlToBase64 } from '@/src/lib/ocrCommands';
const base64 = dataUrlToBase64(canvas.toDataURL('image/png'));
```

#### GET /health
- 응답: `{ "status": "ok", "engine": "RapidOCR" }`

```bash
curl http://127.0.0.1:39835/health
```

#### POST /shutdown
- 서버를 우아하게 종료

```bash
curl -X POST http://127.0.0.1:39835/shutdown
```

---

### 동작 개요
- 시작 시 OCR 엔진 4개를 병렬 로드(약 24MB × 4)
- 요청마다 라운드로빈으로 엔진 배정 → 스레드풀에서 실행
- 응답은 감지 박스와 합쳐진 `raw_text`를 제공하고, 세부 파싱은 Rust에서 수행
