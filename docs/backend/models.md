### 백엔드 모델(Models)

Rust에서 직렬화/역직렬화되어 프론트엔드와 IPC로 교환되는 주요 타입입니다.

---

### ROI
`src-tauri/src/models/roi.rs`
- `Roi { x: i32, y: i32, width: u32, height: u32 }`
- 유틸: `new`, `from_bounds`, `is_valid`, `area`, `contains`, `intersects`

---

### EXP 데이터/세션/통계
`src-tauri/src/models/exp_data.rs`
- `ExpData`(레거시 에일리어스): `{ level:u32, exp:u64, percentage:f64, meso?:u64 }`
- `ExpSnapshot`: 타임스탬프 포함 스냅샷(옵션 HP/MP)
- `ExpSession`: 시작/현재 스냅샷, 스냅샷 목록, 경과시간
- `ExpStats`:
  - `total_exp`, `total_percentage`, `total_meso`
  - `elapsed_seconds`, `exp_per_hour`, `percentage_per_hour`, `meso_per_hour`, `exp_per_minute`
  - `current_level`, `start_level`, `levels_gained`
  - `hp_potions_used`, `mp_potions_used`, `hp_potions_per_minute`, `mp_potions_per_minute`

---

### OCR 결과
`src-tauri/src/models/ocr_result.rs`
- `LevelResult { level:u32, raw_text:String }`
- `ExpResult { absolute:u64, percentage:f64, raw_text:String }`
- `MapResult { map_name:String, raw_text:String }`
- `CombinedOcrResult { level?:LevelResult, exp?:ExpResult, hp?:u32, mp?:u32 }`

---

### 설정(Config)
`src-tauri/src/models/config.rs`
- 창: `WindowDimensions`, `WindowMode('compact'|'dashboard')`, `WindowConfig`
- ROI: `RoiConfig { level?, exp?, hp?, mp? }`
- 트래킹: `TrackingConfig { update_interval, track_meso, auto_start, auto_pause_threshold }`
- 표시: `DisplayConfig { time_format('12h'|'24h'), number_format, show_expected_time, graph_time_window, show_trend_line }`
- 오디오: `AudioConfig { volume, enable_sounds, level_up_sound, milestone_sound }`
- 고급: `AdvancedConfig { ocr_engine:'native', preprocessing:{ scale_factor, apply_blur, blur_radius }, spike_threshold, data_retention_days }`
- 최상위: `AppConfig { window, roi, tracking, display, audio, advanced }`
