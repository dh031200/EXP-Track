import { useState } from 'react';
import { useTimerSettingsStore, AverageInterval, AutoStopInterval, AverageCalculationMode } from '../stores/timerSettingsStore';
import './TimerSettingsModal.css';

interface TimerSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function TimerSettingsModal({ isOpen, onClose }: TimerSettingsModalProps) {
  const [currentPage, setCurrentPage] = useState(0);

  const {
    selectedAverageInterval,
    autoStopInterval,
    averageCalculationMode,
    showTotalTime,
    showSessionCount,
    setAverageInterval,
    setAutoStopInterval,
    setAverageCalculationMode,
    toggleTotalTime,
    toggleSessionCount,
    resetToDefaults,
  } = useTimerSettingsStore();

  if (!isOpen) return null;

  return (
    <>
      <div className="modal-backdrop" onClick={onClose} />

      <div className="modal-container timer-settings-modal">
        <div className="modal-header">
          <h2>타이머 설정</h2>
          <button
            className="modal-close-btn"
            onClick={onClose}
            title="닫기"
          >
            ×
          </button>
        </div>

        <div className="modal-content">
          {currentPage === 0 && (
            <div className="settings-section">
              <h3 className="settings-section-title">평균 경험치 표시</h3>
              <p className="settings-section-desc">
                메인 화면에 표시할 평균 경험치 구간을 선택하세요
              </p>

              <div className="interval-options">
                <label className={`interval-option ${selectedAverageInterval === 'none' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="averageInterval"
                    value="none"
                    checked={selectedAverageInterval === 'none'}
                    onChange={() => setAverageInterval('none')}
                  />
                  <span>안함</span>
                </label>

                <label className={`interval-option ${selectedAverageInterval === '1min' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="averageInterval"
                    value="1min"
                    checked={selectedAverageInterval === '1min'}
                    onChange={() => setAverageInterval('1min')}
                  />
                  <span>1분</span>
                </label>

                <label className={`interval-option ${selectedAverageInterval === '5min' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="averageInterval"
                    value="5min"
                    checked={selectedAverageInterval === '5min'}
                    onChange={() => setAverageInterval('5min')}
                  />
                  <span>5분</span>
                </label>

                <label className={`interval-option ${selectedAverageInterval === '10min' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="averageInterval"
                    value="10min"
                    checked={selectedAverageInterval === '10min'}
                    onChange={() => setAverageInterval('10min')}
                  />
                  <span>10분</span>
                </label>

                <label className={`interval-option ${selectedAverageInterval === '30min' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="averageInterval"
                    value="30min"
                    checked={selectedAverageInterval === '30min'}
                    onChange={() => setAverageInterval('30min')}
                  />
                  <span>30분</span>
                </label>

                <label className={`interval-option ${selectedAverageInterval === '1hour' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="averageInterval"
                    value="1hour"
                    checked={selectedAverageInterval === '1hour'}
                    onChange={() => setAverageInterval('1hour')}
                  />
                  <span>1시간</span>
                </label>
              </div>
            </div>
          )}

          {currentPage === 1 && (
            <div className="settings-section">
              <h3 className="settings-section-title">타이머 자동 정지</h3>
              <p className="settings-section-desc">
                설정한 시간이 지나면 자동으로 추적을 중지합니다
              </p>

              <div className="interval-options">
                <label className={`interval-option ${autoStopInterval === 'none' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="autoStopInterval"
                    value="none"
                    checked={autoStopInterval === 'none'}
                    onChange={() => setAutoStopInterval('none')}
                  />
                  <span>안함</span>
                </label>

                <label className={`interval-option ${autoStopInterval === '5min' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="autoStopInterval"
                    value="5min"
                    checked={autoStopInterval === '5min'}
                    onChange={() => setAutoStopInterval('5min')}
                  />
                  <span>5분</span>
                </label>

                <label className={`interval-option ${autoStopInterval === '15min' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="autoStopInterval"
                    value="15min"
                    checked={autoStopInterval === '15min'}
                    onChange={() => setAutoStopInterval('15min')}
                  />
                  <span>15분</span>
                </label>

                <label className={`interval-option ${autoStopInterval === '30min' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="autoStopInterval"
                    value="30min"
                    checked={autoStopInterval === '30min'}
                    onChange={() => setAutoStopInterval('30min')}
                  />
                  <span>30분</span>
                </label>

                <label className={`interval-option ${autoStopInterval === '1hour' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="autoStopInterval"
                    value="1hour"
                    checked={autoStopInterval === '1hour'}
                    onChange={() => setAutoStopInterval('1hour')}
                  />
                  <span>1시간</span>
                </label>
              </div>
            </div>
          )}

          {currentPage === 2 && (
            <div className="settings-section">
              <h3 className="settings-section-title">평균 계산 방식</h3>
              <p className="settings-section-desc">
                평균 경험치 계산 방식을 선택하세요
              </p>

              <div className="interval-options">
                <label className={`interval-option ${averageCalculationMode === 'prediction' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="calculationMode"
                    value="prediction"
                    checked={averageCalculationMode === 'prediction'}
                    onChange={() => setAverageCalculationMode('prediction')}
                  />
                  <div className="option-content">
                    <span className="option-title">[예상] 경험치</span>
                    <span className="option-desc">초반 데이터로 예측</span>
                  </div>
                </label>

                <label className={`interval-option ${averageCalculationMode === 'per_interval' ? 'selected' : ''}`}>
                  <input
                    type="radio"
                    name="calculationMode"
                    value="per_interval"
                    checked={averageCalculationMode === 'per_interval'}
                    onChange={() => setAverageCalculationMode('per_interval')}
                  />
                  <div className="option-content">
                    <span className="option-title">[분당] 경험치</span>
                    <span className="option-desc">최근 N분 실제 데이터</span>
                  </div>
                </label>
              </div>
            </div>
          )}

          <div className="pagination-dots">
            <span
              className={currentPage === 0 ? 'active' : ''}
              onClick={() => setCurrentPage(0)}
            />
            <span
              className={currentPage === 1 ? 'active' : ''}
              onClick={() => setCurrentPage(1)}
            />
            <span
              className={currentPage === 2 ? 'active' : ''}
              onClick={() => setCurrentPage(2)}
            />
          </div>
        </div>
      </div>
    </>
  );
}
