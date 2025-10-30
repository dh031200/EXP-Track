### 백엔드 서비스(Services) 개요

구현 상세는 각 소스 파일을 참고하세요. 여기서는 용도와 흐름만 요약합니다.

---

### 화면 캡처 `services/screen_capture.rs`
- 기본/전체/영역 캡처 제공
- PNG 바이트로 변환 유틸 포함

### OCR 클라이언트 `services/ocr/http_ocr.rs`
- Python FastAPI 서버(`/ocr`, `/health`)와 HTTP 통신
- Level/EXP/HP/MP 인식 호출 메서드 제공

### OCR 트래커 `services/ocr_tracker.rs`
- 4개 OCR 작업(레벨/경험치/HP/MP)을 비동기 병렬로 운용
- 추적 시작/정지/리셋 및 통계 제공
- 이벤트 전송(프론트 훅이 listen으로 구독)

### EXP 계산기 `services/exp_calculator.rs`
- `ExpData/ExpSnapshot` 기반 누적 통계 산출
- 시간당/분당 지표, 레벨/퍼센트 누적, 포션 사용량 집계

### Python 서버 매니저 `services/python_server.rs`
- 앱 시작 시 Python OCR 서버 기동, 종료 시 정리
- 상태 공유를 위해 `tokio::sync::Mutex`로 관리

### 설정 매니저 `services/config.rs`
- `AppConfig`(JSON) 로드/저장
- ROI 개별 저장/삭제 및 전체 조회 지원
