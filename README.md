# EXP Tracker

메이플랜드에서 경험치와 메소 획득량을 추적하는 현대적인 데스크톱 애플리케이션입니다. Tauri 2.x, Rust, React로 제작되었습니다.

## 주요 기능

- **실시간 추적**: 경험치, 레벨, 메소 획득량을 실시간으로 모니터링
- **OCR 기반**: 자동 화면 캡처 및 텍스트 인식
- **시간 평균**: 시간당 획득량 계산, 사용자 정의 가능한 시간 창
- **크로스 플랫폼**: Windows, macOS, Linux 지원
- **경량**: Tauri 기반 ~5-10MB 번들 크기
- **높은 정확도**: PaddleOCR 엔진을 통한 우수한 텍스트 인식

## 기술 스택

### 백엔드 (Rust)
- **Tauri 2.x**: 데스크톱 애플리케이션용 IPC 프레임워크
- **xcap**: 크로스 플랫폼 화면 캡처
- **image/imageproc**: 이미지 처리 파이프라인
- **PaddleOCR**: 고정확도 OCR 엔진

### 프론트엔드 (React)
- **React 19**: 모던 UI 라이브러리
- **TypeScript 5.8**: 타입 안전 개발
- **Zustand**: 경량 상태 관리
- **Vite 7**: 빠른 빌드 도구

## 개발

### 필수 요구사항
<details>
<summary>개발 환경 요구사항 보기</summary>

- Rust 1.75+ (stable)
- Node.js 18+
- 플랫폼별 의존성 ([Tauri 필수 요구사항](https://tauri.app/v2/guides/prerequisites/) 참조)
</details>

### 설치 및 실행
<details>
<summary>명령어 보기</summary>

```bash
# 의존성 설치
npm install

# 개발 서버 실행
npm run tauri dev
```
</details>

### 빌드
<details>
<summary>빌드 명령어 보기</summary>

```bash
# 프로덕션 빌드
npm run tauri build

# 플랫폼별 빌드
npm run tauri build -- --target x86_64-pc-windows-msvc   # Windows
npm run tauri build -- --target aarch64-apple-darwin     # macOS (Apple Silicon)
npm run tauri build -- --target x86_64-unknown-linux-gnu # Linux
```
</details>

## 프로젝트 구조

```
exp-tracker/
├── src/              # React 프론트엔드
│   ├── components/   # UI 컴포넌트
│   ├── stores/       # Zustand 상태 관리
│   └── utils/        # 헬퍼 함수
├── src-tauri/        # Rust 백엔드
│   ├── commands/     # Tauri 명령어 (IPC)
│   ├── services/     # 비즈니스 로직
│   ├── models/       # 데이터 구조
│   └── utils/        # 유틸리티
└── tests/            # 통합 테스트
```

## 라이선스

MIT License - 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.
