# EXP Tracker

메이플랜드에서 사냥 효율을 실시간으로 추적하는 데스크톱 애플리케이션입니다. 게임 화면의 특정 영역을 읽어 경험치와 포션 사용량을 자동으로 기록하며, 게임 클라이언트와는 전혀 통신하지 않아 안전하게 사용할 수 있습니다.

## 주요 기능

### 📊 실시간 경험치 추적
- **현재 레벨 & 경험치** — OCR로 자동 인식
- **시간당 경험치** — 실시간 효율 계산
- **레벨업 예상 시간** — 현재 속도 기준 남은 시간 표시
- **구간별 평균** — 1분/5분/10분/30분/1시간 단위 평균 계산

### 💊 포션 사용량 추적
- **HP/MP 포션 개수** — 실시간 모니터링
- **분당 사용량** — 포션 소비 속도 계산
- **포션 비용 계산** — 단가 입력 시 총 비용 산출

### 💰 메소 수익 계산
- **사냥 전후 메소 입력** — 순수 메소 획득량 계산
- **포션 비용 차감** — 순이익 자동 계산
- **실시간 미리보기** — 계산 전 결과 확인 가능

### ⏱️ 세션 관리
- **타이머 기능** — 시작/일시정지/리셋
- **목표 시간 설정** — 원하는 사냥 시간 지정 가능
- **히스토리 기록** — 과거 사냥 세션 저장 및 조회 (최대 100개)

### ⚙️ 편의 기능
- **투명도 조절** — 게임 화면 가리지 않도록 설정
- **항상 위 표시** — 게임 중에도 확인 가능
- **글로벌 단축키** — <kbd>`</kbd> (백틱) 키로 빠른 시작/정지
- **ROI 영역 설정** — 드래그로 쉽게 인식 영역 지정

## 다운로드

| OS | 다운로드 링크 |
|---|---|
| **macOS** (Apple Silicon) | [DMG 파일](https://github.com/dh031200/EXP-Track/releases/latest) |
| **Windows** | 준비 중 |

> **참고:** Windows 버전은 개발 중입니다. 직접 빌드하려면 아래 개발자 가이드를 참고하세요.

## 사용 방법

### 1. 프로그램 실행
다운로드한 파일을 설치하고 EXP Tracker를 실행합니다.

### 2. OCR 서버 연결 확인
좌측 상단의 OCR 상태를 확인합니다:
- 🟢 **OCR** — 정상 작동 중
- 🔴 **OCR** — 서버 연결 끊김 (재시작 필요)

### 3. 영역 설정 (ROI)
1. 톱니바퀴 옆의 **영역 설정** 버튼 클릭
2. 각 항목별로 게임 화면의 해당 영역을 드래그로 선택:
   - **레벨** — 캐릭터 레벨 숫자
   - **경험치** — 경험치 바의 숫자 (예: 5509611)
   - **HP 포션** — HP 포션 개수
   - **MP 포션** — MP 포션 개수

### 4. 추적 시작
▶️ 버튼을 눌러 추적을 시작합니다. 
- **일시정지**: ⏸️ 버튼 또는 <kbd>`</kbd> 키
- **리셋**: 🔄 버튼 (세션 초기화)

### 5. 메소 관리 (선택사항)
💰 버튼을 눌러 메소 수익을 계산할 수 있습니다:
1. **시작 메소** — 사냥 시작 시 보유 메소
2. **종료 메소** — 사냥 종료 시 보유 메소
3. **포션 단가** — HP/MP 포션 개당 가격
4. **계산** 버튼으로 미리보기 → **저장**

---

## 개발자 가이드

### 기술 스택

| 영역 | 기술 |
|---|---|
| **프론트엔드** | React 19 + TypeScript + Zustand |
| **백엔드** | Rust + Tauri 2.x |
| **OCR 엔진** | Python + PaddleOCR (HTTP 서버) |
| **화면 캡처** | xcap 0.7 |
| **상태 관리** | Zustand (persist middleware) |

### 아키텍처 개요

```
┌─────────────────┐
│  React Frontend │ ◄─┐
│  (TypeScript)   │   │
└────────┬────────┘   │
         │            │
         │ IPC        │ Events
         │            │
┌────────▼────────┐   │
│  Tauri Core     │───┘
│  (Rust)         │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼───┐ ┌──▼──────┐
│ Screen│ │ Python  │
│Capture│ │OCR Server│
│(xcap) │ │(Paddle) │
└───────┘ └─────────┘
```

**핵심 컴포넌트:**
1. **OcrTracker** (Rust) — 4개의 독립적인 병렬 OCR 루프 (Level/EXP/HP/MP)
2. **ExpCalculator** (Rust) — 경험치 통계 계산 및 시간당 효율 산출
3. **PotionCalculator** (Rust) — HP/MP 포션 사용량 및 분당 소비율 계산
4. **Python OCR Server** — PaddleOCR 기반 텍스트 인식 (HTTP API)

### 필수 요구사항

- **Rust** 1.75+ (stable)
- **Node.js** 18+
- **Python** 3.10+
- **플랫폼별 빌드 도구** — [Tauri Prerequisites](https://tauri.app/v2/guides/prerequisites/) 참고

### 개발 환경 설정

#### macOS

```bash
# 프로젝트 클론
git clone https://github.com/dh031200/EXP-Track.git
cd EXP-Track

# Node.js 의존성 설치
npm install

# Python OCR 서버 설정
cd python_ocr_server
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
cd ..

# 개발 서버 실행 (Tauri + Vite + Python OCR)
npm run tauri dev
```

#### Windows

```cmd
# 프로젝트 클론
git clone https://github.com/dh031200/EXP-Track.git
cd EXP-Track

# 자동 환경 설정 스크립트 (Node.js, Rust, Python, Build Tools)
# 최초 실행: y,y,y,y 입력 / 이후: N,N 입력
.\scripts\build_python_server.bat

# Node.js 의존성 설치
npm install

# 아이콘 생성
npm run icon:generate

# 개발 서버 실행
npm run tauri dev
```

### 빌드

#### 프로덕션 빌드

```bash
npm run tauri build
```

#### 플랫폼별 타겟 빌드

```bash
# macOS (Apple Silicon)
npm run tauri build -- --target aarch64-apple-darwin

# macOS (Intel)
npm run tauri build -- --target x86_64-apple-darwin

# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

### 프로젝트 구조

```
EXP-Track/
├── src/                          # React Frontend
│   ├── components/               # UI Components
│   │   ├── CompactRoiManager.tsx # ROI 설정 UI
│   │   ├── ExpTrackerDisplay.tsx # 경험치 표시
│   │   ├── HistoryDashboard.tsx  # 세션 히스토리
│   │   └── Settings.tsx          # 설정 UI
│   ├── hooks/                    # Custom Hooks
│   │   ├── useExpTracker.ts
│   │   └── useParallelOcrTracker.ts # 병렬 OCR 트래킹
│   ├── stores/                   # Zustand Stores
│   │   ├── roiStore.ts           # ROI 상태
│   │   ├── sessionStore.ts       # 세션 기록
│   │   ├── trackingStore.ts      # 트래킹 상태
│   │   └── mesoStore.ts          # 메소 계산
│   └── lib/                      # Tauri Commands API
│       ├── expCommands.ts
│       ├── ocrCommands.ts
│       └── trackingCommands.ts
│
├── src-tauri/                    # Rust Backend
│   ├── src/
│   │   ├── commands/             # Tauri IPC Commands
│   │   │   ├── tracking.rs       # OCR 트래킹 명령
│   │   │   ├── exp.rs            # 경험치 계산 명령
│   │   │   ├── ocr.rs            # OCR 서비스
│   │   │   └── screen_capture.rs # 화면 캡처
│   │   ├── services/             # Business Logic
│   │   │   ├── ocr_tracker.rs    # 병렬 OCR 트래커
│   │   │   ├── exp_calculator.rs # 경험치 통계 계산
│   │   │   ├── hp_potion_calculator.rs
│   │   │   ├── mp_potion_calculator.rs
│   │   │   └── python_server.rs  # Python OCR 서버 관리
│   │   └── models/               # Data Models
│   │       ├── roi.rs            # ROI 데이터 구조
│   │       ├── exp_data.rs       # 경험치 데이터
│   │       └── ocr_result.rs     # OCR 결과
│   └── resources/                # 번들 리소스
│       └── ocr_server/           # Python OCR 서버 (빌드 시 포함)
│
├── python_ocr_server/            # Python OCR HTTP Server
│   ├── main.py                   # FastAPI 서버
│   └── requirements.txt          # PaddleOCR 의존성
│
└── tests/                        # 테스트 픽스처
    └── fixtures/                 # OCR 테스트용 이미지
```

### 주요 구현 세부사항

#### 1. 병렬 OCR 트래킹 (Rust)

`OcrTracker`는 4개의 독립적인 비동기 루프를 병렬로 실행합니다:
- **Level OCR** — 500ms 간격으로 레벨 인식 (안정성 체크: 2회 연속 일치 필요)
- **EXP OCR** — 500ms 간격으로 경험치 인식 및 `ExpCalculator` 업데이트
- **HP Potion OCR** — 500ms 간격으로 HP 포션 개수 인식
- **MP Potion OCR** — 500ms 간격으로 MP 포션 개수 인식

각 루프는 이미지 캐싱을 통해 중복 OCR 방지:
- 이전 프레임과 동일한 이미지는 스킵
- 변경 감지 시에만 OCR 실행

#### 2. 경험치 계산 (Rust)

`ExpCalculator`는 메이플랜드 공식 경험치 테이블을 기반으로 통계를 계산합니다:
- 레벨 1~200 경험치 테이블 내장
- 세션 시작 시 초기 데이터 저장
- 각 업데이트마다 총 획득 경험치, 시간당 경험치, 레벨업 예상 시간 계산

#### 3. Python OCR 서버

Tauri 앱 시작 시 자동으로 Python 서버를 실행합니다:
- **자동 시작**: `python_server.rs`에서 서브프로세스로 실행
- **포트**: `localhost:8000`
- **엔드포인트**:
  - `POST /ocr/level` — 레벨 인식
  - `POST /ocr/exp` — 경험치 인식
  - `POST /ocr/hp_potion` — HP 포션 개수 인식
  - `POST /ocr/mp_potion` — MP 포션 개수 인식
  - `GET /health` — 서버 상태 확인

#### 4. 상태 관리 (Zustand)

Zustand persist middleware를 사용하여 로컬스토리지에 상태 저장:
- `roiStore` — ROI 설정 (localStorage)
- `sessionStore` — 세션 히스토리 (최대 100개)
- `mesoStore` — 메소 계산 설정
- `settingsStore` — 투명도, 목표 시간 등

#### 5. 글로벌 단축키

Tauri의 `global-shortcut` 플러그인 사용:
- <kbd>`</kbd> (백틱) 키로 트래킹 시작/일시정지
- ROI 설정 중이거나 설정 화면에서는 비활성화

### 테스트

```bash
# 단위 테스트 실행
npm test

# 테스트 UI
npm run test:ui

# 커버리지
npm run test:coverage
```

### 디버깅

개발 모드에서 실행 시 Rust 백엔드의 디버그 로그가 터미널에 출력됩니다:
- `✅` — 성공
- `❌` — 에러
- `🚀` — 시작
- `⏹️` — 종료
- `⚡` — 이벤트

프론트엔드 DevTools는 `Ctrl+Shift+I` (Windows/Linux) 또는 `Cmd+Option+I` (macOS)로 열 수 있습니다.

### 빌드 최적화

개발 모드에서도 OCR 성능을 위해 최적화가 적용됩니다:

```toml
# Cargo.toml
[profile.dev]
opt-level = 3
```

이를 통해 `tauri dev` 실행 시에도 릴리스 수준의 성능을 제공합니다.

### 기여하기

이슈, 기능 제안, 풀 리퀘스트를 환영합니다!

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### 라이선스

MIT License — 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

---

## FAQ

**Q: OCR 인식률이 낮아요.**  
A: 다음을 확인해보세요:
- ROI 영역이 숫자만 포함하도록 정확히 설정했는지 확인
- 게임 화면 해상도가 너무 낮지 않은지 확인
- 게임 UI 테마가 기본 설정인지 확인

**Q: 프로그램이 느려요.**  
A: 다음을 시도해보세요:
- 다른 화면 캡처 프로그램 종료 (OBS, Discord 등)
- ROI 영역을 필요한 최소 크기로 설정
- 백그라운드 프로그램 최소화

**Q: Python OCR 서버가 시작되지 않아요.**  
A: Python 3.10+ 설치 여부를 확인하고, 수동으로 서버를 실행해보세요:
```bash
cd python_ocr_server
source .venv/bin/activate  # Windows: .venv\Scripts\activate
python main.py
```

**Q: 글로벌 단축키가 작동하지 않아요.**  
A: macOS에서는 접근성 권한이 필요합니다:
- 시스템 환경설정 > 보안 및 개인 정보 보호 > 개인 정보 보호 > 접근성
- EXP Tracker 추가
