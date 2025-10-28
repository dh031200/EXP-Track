import { useSettingsStore } from '../stores/settingsStore';
import './Settings.css';

export function Settings() {
  const { backgroundOpacity, targetDuration, setBackgroundOpacity, setTargetDuration, resetSettings } = useSettingsStore();

  const handleOpacityChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setBackgroundOpacity(parseFloat(e.target.value));
  };

  const handleDurationChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTargetDuration(parseInt(e.target.value));
  };

  const opacityPercent = Math.round(backgroundOpacity * 100);

  return (
    <div className="settings-container">
      <h2>설정</h2>

      <div className="settings-section">
        <h3>외형</h3>

        <div className="settings-item">
          <label htmlFor="opacity-slider" className="settings-label">
            배경 투명도: <strong>{opacityPercent}%</strong>
          </label>
          <div className="slider-container">
            <span className="slider-label">30%</span>
            <input
              id="opacity-slider"
              type="range"
              min="0.3"
              max="1"
              step="0.05"
              value={backgroundOpacity}
              onChange={handleOpacityChange}
              className="opacity-slider"
            />
            <span className="slider-label">100%</span>
          </div>
          <p className="settings-help">
            앱 전체의 투명도를 조절합니다. 값이 낮을수록 더 투명해집니다.
          </p>
        </div>
      </div>

      <div className="settings-section">
        <h3>타이머</h3>

        <div className="settings-item">
          <label htmlFor="duration-select" className="settings-label">
            목표 시간 설정
          </label>
          <select
            id="duration-select"
            value={targetDuration}
            onChange={handleDurationChange}
            className="duration-select"
          >
            <option value="0">사용 안 함</option>
            <option value="5">5분</option>
            <option value="10">10분</option>
            <option value="15">15분</option>
            <option value="30">30분</option>
            <option value="60">1시간</option>
            <option value="120">2시간</option>
            <option value="180">3시간</option>
          </select>
          <p className="settings-help">
            목표 시간을 설정하면 타이머에 완료 예정 시각이 표시됩니다.
          </p>
        </div>
      </div>

      {/* <div className="settings-actions">
        <button onClick={resetSettings} className="reset-button">
          기본값으로 재설정
        </button>
      </div> */}
    </div>
  );
}
