# Build Scripts

## Python OCR Server Bundling

### Development Mode
개발 모드에서는 Python 서버가 자동으로 `.venv/bin/python`을 사용하여 시작됩니다.

별도로 빌드할 필요 없이:
```bash
npm run tauri dev
```

### Production Build

프로덕션 빌드 전에 Python 서버를 번들링해야 합니다:

```bash
# 1. Python 서버 번들링
./scripts/build_python_server.sh

# 2. Tauri 앱 빌드
npm run tauri build
```

#### 상세 과정:

1. **Python 서버 번들링**
   - PyInstaller로 Python 서버를 단일 실행파일로 패키징
   - RapidOCR 모델 포함
   - `src-tauri/resources/ocr_server`에 복사

2. **Tauri 빌드**
   - `tauri.conf.json`의 resources 설정에 따라 번들에 포함
   - Rust 코드가 프로덕션 모드에서 `resources/ocr_server` 실행

#### 주의사항:

- ⚠️ PyInstaller 빌드는 시간이 오래 걸립니다 (3-5분)
- ⚠️ 플랫폼별로 별도 빌드 필요 (macOS, Windows, Linux)
- ⚠️ Python 서버 코드 변경시 재빌드 필요

#### 빌드 확인:

```bash
# Python 서버 바이너리 확인
ls -lh src-tauri/resources/ocr_server

# 프로덕션 빌드 실행
./src-tauri/target/release/exp-tracker
```

## 개발 vs 프로덕션 차이

| 환경 | Python 실행 방법 | OCR 성능 |
|------|------------------|----------|
| 개발 | `.venv/bin/python main.py` | 130ms |
| 프로덕션 | `resources/ocr_server` (PyInstaller 번들) | 130ms |

프로덕션 번들은:
- ✅ Python 런타임 내장
- ✅ 모든 의존성 포함
- ✅ 단일 파일로 배포 가능
- ✅ 사용자가 Python 설치 불필요
