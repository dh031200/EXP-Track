import React, { useEffect, useState } from 'react';
import { useSettingsStore } from '../stores/settingsStore';
import { loadAppConfig, saveAppConfig, AppConfig } from '../lib/configCommands';
import './Settings.css';

export function Settings() {
  const { backgroundOpacity, targetDuration, setBackgroundOpacity, setTargetDuration } = useSettingsStore();
  const [appConfig, setAppConfig] = useState<AppConfig | null>(null);
  const [scanInterval, setScanInterval] = useState(1);

  useEffect(() => {
    loadAppConfig().then(config => {
      setAppConfig(config);
      setScanInterval(config.tracking.update_interval);
    }).catch(err => {
      console.error('Failed to load config:', err);
    });
  }, []);

  const handleOpacityChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setBackgroundOpacity(parseFloat(e.target.value));
  };

  const handleDurationChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTargetDuration(parseInt(e.target.value));
  };

  const handleIntervalChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = parseInt(e.target.value);
    setScanInterval(val);
    if (appConfig) {
      const newConfig = { 
        ...appConfig, 
        tracking: { 
          ...appConfig.tracking, 
          update_interval: val 
        } 
      };
      setAppConfig(newConfig);
      await saveAppConfig(newConfig);
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
        <h3>인식 설정</h3>
        <div className="settings-item">
          <label htmlFor="interval-slider" className="settings-label">
            인식 주기: <strong>{scanInterval}초</strong>
          </label>
          <div className="slider-container">
            <span className="slider-label">1초</span>
            <input
              id="interval-slider"
              type="range"
              min="1"
              max="10"
              step="1"
              value={scanInterval}
              onChange={handleIntervalChange}
              className="opacity-slider" // Reusing style
            />
            <span className="slider-label">10초</span>
          </div>
          <p className="settings-help">
            경험치와 레벨을 인식하는 주기를 설정합니다. (기본값: 1초)
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
    </div>
  );
}
