import { useState, useEffect } from 'react';
import { useSettingsStore } from '../stores/settingsStore';
import { getPotionSlotConfig, setPotionSlotConfig, VALID_SLOTS } from '../lib/configCommands';
import './Settings.css';

export function Settings() {
  const { backgroundOpacity, targetDuration, setBackgroundOpacity, setTargetDuration, resetSettings } = useSettingsStore();

  // Potion slot configuration state
  const [hpSlot, setHpSlot] = useState<string>('shift');
  const [mpSlot, setMpSlot] = useState<string>('ins');
  const [potionConfigError, setPotionConfigError] = useState<string | null>(null);

  // Load potion config on mount
  useEffect(() => {
    getPotionSlotConfig()
      .then(config => {
        setHpSlot(config.hp_potion_slot);
        setMpSlot(config.mp_potion_slot);
      })
      .catch(err => {
        console.error('Failed to load potion config:', err);
        setPotionConfigError('포션 설정을 불러오는데 실패했습니다');
      });
  }, []);

  const handleOpacityChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setBackgroundOpacity(parseFloat(e.target.value));
  };

  const handleDurationChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTargetDuration(parseInt(e.target.value));
  };

  const handleHpSlotChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newHpSlot = e.target.value;

    if (newHpSlot === mpSlot) {
      setPotionConfigError('HP 포션과 MP 포션은 서로 다른 칸이어야 합니다');
      return;
    }

    try {
      await setPotionSlotConfig(newHpSlot, mpSlot);
      setHpSlot(newHpSlot);
      setPotionConfigError(null);
    } catch (err) {
      console.error('Failed to save HP slot config:', err);
      setPotionConfigError('HP 포션 설정을 저장하는데 실패했습니다');
    }
  };

  const handleMpSlotChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newMpSlot = e.target.value;

    if (newMpSlot === hpSlot) {
      setPotionConfigError('HP 포션과 MP 포션은 서로 다른 칸이어야 합니다');
      return;
    }

    try {
      await setPotionSlotConfig(hpSlot, newMpSlot);
      setMpSlot(newMpSlot);
      setPotionConfigError(null);
    } catch (err) {
      console.error('Failed to save MP slot config:', err);
      setPotionConfigError('MP 포션 설정을 저장하는데 실패했습니다');
    }
  };

  const opacityPercent = Math.round(backgroundOpacity * 100);

  return (
    <div className="settings-container">
      <div className="settings-section">
        <h3>배경</h3>

        <div className="settings-item">
          <label htmlFor="opacity-slider" className="settings-label">
            투명도: <strong>{opacityPercent}%</strong>
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
            시간 설정
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

      <div className="settings-section">
        <h3>포션 설정</h3>

        {potionConfigError && (
          <div className="settings-error">
            {potionConfigError}
          </div>
        )}

        <div className="settings-item">
          <label htmlFor="hp-slot-select" className="settings-label">
            HP 포션 칸
          </label>
          <select
            id="hp-slot-select"
            value={hpSlot}
            onChange={handleHpSlotChange}
            className="duration-select"
          >
            {VALID_SLOTS.map(slot => (
              <option key={slot} value={slot}>
                {slot.toUpperCase()}
              </option>
            ))}
          </select>
          <p className="settings-help">
            HP 포션이 있는 인벤토리 칸을 선택하세요.
          </p>
        </div>

        <div className="settings-item">
          <label htmlFor="mp-slot-select" className="settings-label">
            MP 포션 칸
          </label>
          <select
            id="mp-slot-select"
            value={mpSlot}
            onChange={handleMpSlotChange}
            className="duration-select"
          >
            {VALID_SLOTS.map(slot => (
              <option key={slot} value={slot}>
                {slot.toUpperCase()}
              </option>
            ))}
          </select>
          <p className="settings-help">
            MP 포션이 있는 인벤토리 칸을 선택하세요.
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
