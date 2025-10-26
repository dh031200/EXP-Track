# EXP Tracker

메이플랜드에서 사냥 효율을 체크하고 싶은데 일일이 계산하기 귀찮으셨나요? EXP Tracker는 게임 화면만 보고 자동으로 경험치와 포션 사용량을 추적해주는 프로그램입니다.

게임 클라이언트와는 전혀 통신하지 않고, 단순히 화면에 보이는 숫자를 읽어서 계산합니다. 게임 메모리나 패킷에 접근하지 않기 때문에 안전하게 사용할 수 있습니다.

## 기능 소개

메이플랜드 게임 화면을 읽어서 다음 정보를 자동으로 추적합니다:

- **경험치 추적** — 현재 경험치, 레벨, 시간당 사냥 효율
- **포션 사용량** — HP/MP 포션 소비 현황
- **평균 계산** — 원하는 시간 구간(5분, 10분, 30분, 1시간 등)의 평균 획득량
- **히스토리 기록** — 과거 사냥 세션을 저장하고 비교

## 주요 특징

- ✨ **완전 자동** — 한 번 설정하면 게임하는 동안 알아서 추적
- 📊 **실시간 확인** — 지금 이 순간의 경험치 획득 속도를 바로 볼 수 있음
- 🎯 **간편한 설정** — 드래그로 화면 영역만 지정하면 끝
- 📈 **기록 보관** — 어제 사냥이랑 오늘 사냥 비교 가능
- 🖥️ **크로스 플랫폼** — macOS, Windows 모두 지원 (*Windows는 테스트 중)
- ⚡ **가볍고 안전** — 게임 성능에 전혀 영향 없음

## 사용 방법

간단하게 3단계면 시작할 수 있습니다:

1. **프로그램 실행하기**

   설치 후 EXP Tracker를 켜주세요

2. **영역 지정하기**

   설정에서 "영역 설정" 버튼을 누르고, 게임 화면에서 경험치 바나 포션 수량이 보이는 부분을 드래그해서 선택

3. **추적 시작하기**

   "추적 시작" 버튼만 누르면 자동으로 데이터를 읽기 시작합니다

## 다운로드

> 현재 개발 중입니다. 정식 릴리스는 준비 중이며, 빌드 방법은 아래 기술 정보를 참고하세요.

---

<details>
<summary><strong>🔧 기술 정보 (개발자용)</strong></summary>

### 기술 스택 요약

- **프론트엔드**: React 19 + TypeScript + Zustand
- **백엔드**: Rust + Tauri 2.x
- **OCR 엔진**: PaddleOCR (Python 서버)
- **화면 캡처**: xcap 라이브러리

### 개발 환경 설정

**필수 요구사항**
- Rust 1.75+ (stable)
- Node.js 18+
- Python 3.10+ (OCR 서버용)
- 플랫폼별 의존성: [Tauri 필수 요구사항](https://tauri.app/v2/guides/prerequisites/) 참조

**설치 및 실행**
```bash
# 의존성 설치
npm install

# Python OCR 서버 설정 (최초 1회)
cd python_ocr_server
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -r requirements.txt

# 개발 서버 실행
npm run tauri dev
```

### 빌드

**프로덕션 빌드**
```bash
npm run tauri build
```

**플랫폼별 빌드**
```bash
# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# macOS (Apple Silicon)
npm run tauri build -- --target aarch64-apple-darwin

# macOS (Intel)
npm run tauri build -- --target x86_64-apple-darwin

# Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

### 프로젝트 구조

```
exp-tracker/
├── src/                    # React 프론트엔드
│   ├── components/         # UI 컴포넌트
│   ├── stores/            # Zustand 상태 관리
│   ├── hooks/             # React 커스텀 훅
│   └── services/          # API 통신 레이어
├── src-tauri/             # Rust 백엔드
│   ├── src/
│   │   ├── commands/      # Tauri IPC 명령어
│   │   ├── services/      # 비즈니스 로직
│   │   └── models/        # 데이터 구조
│   └── resources/         # 번들 리소스
├── python_ocr_server/     # OCR 처리 서버
└── tests/                 # 통합 테스트
```

### 아키텍처 개요

1. **Tauri (Rust)**: 화면 캡처 및 애플리케이션 로직
2. **Python OCR 서버**: 이미지에서 텍스트 추출
3. **React 프론트엔드**: 사용자 인터페이스 및 데이터 시각화

</details>

---

## 라이선스

MIT License - 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

## 기여하기

이슈 제보, 기능 제안, 풀 리퀘스트 환영합니다!
