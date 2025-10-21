import { useState, useCallback, useEffect } from "react";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { RoiConfigModal } from "./components/RoiConfigModal";
import { Settings } from "./components/Settings";
import { TimerSettingsModal } from "./components/TimerSettingsModal";
import { useSettingsStore } from "./stores/settingsStore";
import { useRoiStore } from "./stores/roiStore";
import { useTrackingStore } from "./stores/trackingStore";
import { useSessionStore } from "./stores/sessionStore";
import { useTimerSettingsStore } from "./stores/timerSettingsStore";
import "./App.css";

// Import icons
import startIcon from "/icons/start.png";
import pauseIcon from "/icons/pause.png";
import resetIcon from "/icons/reset.png";
import roiIcon from "/icons/roi.png";
import settingIcon from "/icons/setting.png";
import historyIcon from "/icons/history.png";
import timerIcon from "/icons/timer.png";

function App() {
  const [isSelecting, setIsSelecting] = useState(false);
  const [showRoiModal, setShowRoiModal] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showTimerSettings, setShowTimerSettings] = useState(false);

  const backgroundOpacity = useSettingsStore((state) => state.backgroundOpacity);
  const { levelRoi, expRoi, mapLocationRoi } = useRoiStore(); // mesoRoi commented out
  const {
    state: trackingState,
    elapsedSeconds,
    pausedSeconds,
    startTracking,
    pauseTracking,
    resetTracking,
    incrementTimer,
    getActiveDuration
  } = useTrackingStore();

  const {
    sessions,
    startSession,
    endSession,
    updateSessionDuration,
  } = useSessionStore();

  const { selectedAverageInterval } = useTimerSettingsStore();

  // Check if any ROI is configured
  const hasAnyRoi = levelRoi !== null || expRoi !== null || mapLocationRoi !== null;

  // Timer effect - increment every second when tracking
  useEffect(() => {
    if (trackingState === 'tracking') {
      const interval = setInterval(() => {
        incrementTimer();
        // Update current session duration
        const activeDuration = getActiveDuration();
        updateSessionDuration(elapsedSeconds + 1, pausedSeconds);
      }, 1000);
      return () => clearInterval(interval);
    }
  }, [trackingState, incrementTimer, elapsedSeconds, pausedSeconds, getActiveDuration, updateSessionDuration]);

  // Format elapsed seconds as HH:MM:SS
  const formatTime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // Calculate average exp for selected interval
  const calculateAverage = (): { label: string; value: string } | null => {
    if (selectedAverageInterval === 'none' || sessions.length === 0) {
      return null;
    }

    const intervalMinutes = {
      '5min': 5,
      '10min': 10,
      '30min': 30,
      '1hour': 60,
    }[selectedAverageInterval] || 0;

    const intervalLabel = {
      '5min': '5분',
      '10min': '10분',
      '30min': '30분',
      '1hour': '1시간',
    }[selectedAverageInterval] || '';

    const intervalSeconds = intervalMinutes * 60;
    const totalExp = sessions.reduce((sum, s) => sum + (s.expGained || 0), 0);
    const totalDuration = sessions.reduce((sum, s) => sum + s.duration, 0);

    if (totalDuration === 0) {
      return { label: intervalLabel, value: '0' };
    }

    const expPerSecond = totalExp / totalDuration;
    const avgExp = Math.floor(expPerSecond * intervalSeconds);

    return { label: intervalLabel, value: avgExp.toLocaleString() };
  };

  const averageData = calculateAverage();

  const handleSelectingChange = useCallback((selecting: boolean) => {
    setIsSelecting(selecting);
  }, []);

  const handleToggleTracking = () => {
    if (!hasAnyRoi) {
      setShowRoiModal(true);
      return;
    }

    if (trackingState === 'idle') {
      // Start new session
      startSession();
      startTracking();
      // TODO: Start OCR and exp recording
    } else if (trackingState === 'paused') {
      // Resume tracking
      startTracking();
      // TODO: Resume OCR and exp recording
    } else if (trackingState === 'tracking') {
      // Pause tracking
      pauseTracking();
      // TODO: Pause OCR and exp recording
    }
  };

  const handleReset = () => {
    if (trackingState !== 'idle') {
      // Save session to history before resetting
      endSession();
    }
    resetTracking();
    // TODO: Clear exp data and reset to initial state
  };

  const handleClose = async () => {
    const window = getCurrentWindow();
    await window.close();
  };

  const handleMinimize = async () => {
    const window = getCurrentWindow();
    await window.minimize();
  };

  const handleDragStart = async (e: React.MouseEvent) => {
    e.preventDefault();
    const window = getCurrentWindow();
    await window.startDragging();
  };

  const handleOpenHistory = async () => {
    // Check if window already exists (getByLabel is async in Tauri 2.x)
    const existingWindow = await WebviewWindow.getByLabel('history');
    if (existingWindow) {
      await existingWindow.focus(); // use focus() instead of setFocus() in Tauri 2.x
      return;
    }

    // Create new window
    const historyWindow = new WebviewWindow('history', {
      url: '/history',
      title: '사냥 기록',
      width: 920,
      height: 720,
      resizable: true,
      center: true,
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
    });

    // Wait for window to be ready
    historyWindow.once('tauri://created', () => {
      console.log('History window created');
    });

    historyWindow.once('tauri://error', (e) => {
      console.error('Error creating history window:', e);
    });
  };

  return (
    <div
      style={{
        width: '100vw',
        height: '100vh',
        background: 'transparent',
        overflow: 'hidden',
        borderRadius: isSelecting ? '0' : '12px',
        display: 'flex',
        flexDirection: 'column',
        // Apply opacity to entire app (all elements together)
        opacity: isSelecting ? 1 : backgroundOpacity,
        position: 'relative'
      }}
    >
      {/* Titlebar with integrated controls */}
      {!isSelecting && (
        <div
          onMouseDown={handleDragStart}
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            height: '44px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            background: 'rgba(255, 255, 255, 0.98)',
            borderTopLeftRadius: '12px',
            borderTopRightRadius: '12px',
            borderBottom: '1px solid rgba(0, 0, 0, 0.08)',
            cursor: 'grab',
            userSelect: 'none',
          }}
        >
          {/* Title text */}
          <div style={{
            fontSize: '12px',
            fontWeight: '500',
            color: 'rgba(0, 0, 0, 0.5)',
            pointerEvents: 'none'
          }}>
            EXP Tracker
          </div>

          {/* Window controls - prevent drag on click */}
          <div
            onMouseDown={(e) => e.stopPropagation()}
            style={{
              position: 'absolute',
              top: '6px',
              right: '12px',
              display: 'flex',
              gap: '8px',
            }}
          >
            <button
              onClick={handleMinimize}
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '8px',
                border: 'none',
                background: 'rgba(0, 0, 0, 0.4)',
                color: '#fff',
                fontSize: '20px',
                fontWeight: '300',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                transition: 'all 0.15s ease',
                paddingBottom: '4px',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = 'rgba(0, 0, 0, 0.6)';
                e.currentTarget.style.transform = 'scale(1.05)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'rgba(0, 0, 0, 0.4)';
                e.currentTarget.style.transform = 'scale(1)';
              }}
              title="Minimize"
            >
              −
            </button>
            <button
              onClick={handleClose}
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '8px',
                border: 'none',
                background: 'rgba(255, 59, 48, 0.8)',
                color: '#fff',
                fontSize: '20px',
                fontWeight: '300',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                transition: 'all 0.15s ease',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = '#ff3b30';
                e.currentTarget.style.transform = 'scale(1.05)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'rgba(255, 59, 48, 0.8)';
                e.currentTarget.style.transform = 'scale(1)';
              }}
              title="Close"
            >
              ×
            </button>
          </div>
        </div>
      )}

      <main className="container" style={{
        background: isSelecting ? 'transparent' : 'rgba(255, 255, 255, 0.98)',
        marginTop: isSelecting ? '0' : '44px',
        padding: isSelecting ? '0' : '16px',
        height: isSelecting ? '100%' : 'calc(100% - 44px)',
        borderBottomLeftRadius: isSelecting ? '0' : '12px',
        borderBottomRightRadius: isSelecting ? '0' : '12px',
        overflow: isSelecting ? 'hidden' : 'hidden',
        boxSizing: 'border-box',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '12px'
      }}>
        {!isSelecting && !showSettings && (
          <>
            {/* Central controls: Start/Pause toggle + Timer */}
            <div style={{
              display: 'flex',
              alignItems: 'center',
              gap: '20px',
              justifyContent: 'center'
            }}>
              {/* Start/Pause Toggle Button */}
              <button
                onClick={handleToggleTracking}
                disabled={!hasAnyRoi}
                style={{
                  width: '64px',
                  height: '64px',
                  borderRadius: '12px',
                  border: 'none',
                  background: !hasAnyRoi
                    ? 'rgba(0, 0, 0, 0.1)'
                    : trackingState === 'tracking'
                      ? 'linear-gradient(135deg, #FF9800 0%, #F57C00 100%)'
                      : 'linear-gradient(135deg, #4CAF50 0%, #45a049 100%)',
                  cursor: hasAnyRoi ? 'pointer' : 'not-allowed',
                  transition: 'all 0.2s ease',
                  boxShadow: hasAnyRoi ? '0 4px 12px rgba(0, 0, 0, 0.15)' : 'none',
                  padding: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  opacity: hasAnyRoi ? 1 : 0.5
                }}
                onMouseEnter={(e) => {
                  if (hasAnyRoi) {
                    e.currentTarget.style.transform = 'scale(1.05)';
                    e.currentTarget.style.boxShadow = '0 6px 16px rgba(0, 0, 0, 0.2)';
                  }
                }}
                onMouseLeave={(e) => {
                  if (hasAnyRoi) {
                    e.currentTarget.style.transform = 'scale(1)';
                    e.currentTarget.style.boxShadow = '0 4px 12px rgba(0, 0, 0, 0.15)';
                  }
                }}
                title={!hasAnyRoi ? 'ROI 설정 필요' : trackingState === 'tracking' ? '일시정지' : '시작'}
              >
                <img
                  src={trackingState === 'tracking' ? pauseIcon : startIcon}
                  alt={trackingState === 'tracking' ? 'Pause' : 'Start'}
                  style={{ width: '36px', height: '36px' }}
                />
              </button>

              {/* Timer Display - Much Larger */}
              <div style={{
                fontSize: '56px',
                fontWeight: '700',
                color: trackingState === 'tracking' ? '#4CAF50' : '#666',
                fontFamily: 'monospace',
                letterSpacing: '2px',
                textAlign: 'center'
              }}>
                {formatTime(elapsedSeconds)}
              </div>
            </div>

            {/* Status text */}
            <div style={{
              fontSize: '12px',
              color: !hasAnyRoi ? '#FF9800' : trackingState === 'tracking' ? '#4CAF50' : '#666',
              fontWeight: '500',
              textAlign: 'center',
              marginTop: '8px'
            }}>
              {!hasAnyRoi ? 'ROI 설정 필요' : trackingState === 'tracking' ? '추적 중...' : trackingState === 'paused' ? '일시정지됨' : '준비됨'}
            </div>

            {/* Average EXP Display */}
            {averageData && (
              <div style={{
                marginTop: '4px',
                padding: '6px 12px',
                background: 'rgba(102, 126, 234, 0.08)',
                border: '1px solid rgba(102, 126, 234, 0.2)',
                borderRadius: '6px',
                display: 'inline-block'
              }}>
                <div style={{
                  fontSize: '10px',
                  color: '#667eea',
                  fontWeight: '600',
                  marginBottom: '2px',
                  letterSpacing: '0.5px'
                }}>
                  평균 ({averageData.label})
                </div>
                <div style={{
                  fontSize: '14px',
                  color: '#667eea',
                  fontWeight: '700',
                  fontFamily: 'monospace'
                }}>
                  {averageData.value} 경험치
                </div>
              </div>
            )}

            {/* Bottom-left reset button */}
            <div style={{
              position: 'absolute',
              bottom: '16px',
              left: '16px'
            }}>
              <button
                onClick={handleReset}
                disabled={trackingState === 'idle'}
                style={{
                  width: '48px',
                  height: '48px',
                  borderRadius: '8px',
                  border: '1px solid rgba(0, 0, 0, 0.1)',
                  background: 'rgba(0, 0, 0, 0.05)',
                  cursor: trackingState !== 'idle' ? 'pointer' : 'not-allowed',
                  transition: 'all 0.15s ease',
                  padding: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  opacity: trackingState !== 'idle' ? 1 : 0.3
                }}
                onMouseEnter={(e) => {
                  if (trackingState !== 'idle') {
                    e.currentTarget.style.background = 'rgba(0, 0, 0, 0.08)';
                    e.currentTarget.style.transform = 'scale(1.05)';
                  }
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.05)';
                  e.currentTarget.style.transform = 'scale(1)';
                }}
                title="리셋"
              >
                <img
                  src={resetIcon}
                  alt="Reset"
                  style={{ width: '24px', height: '24px' }}
                />
              </button>
            </div>

            {/* Bottom-right menu buttons (Timer -> History -> ROI -> Settings) */}
            <div style={{
              position: 'absolute',
              bottom: '16px',
              right: '16px',
              display: 'flex',
              gap: '6px'
            }}>
              <button
                onClick={() => setShowTimerSettings(true)}
                style={{
                  width: '40px',
                  height: '40px',
                  background: 'rgba(0, 0, 0, 0.05)',
                  border: '1px solid rgba(0, 0, 0, 0.1)',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                  padding: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center'
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.08)';
                  e.currentTarget.style.transform = 'scale(1.05)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.05)';
                  e.currentTarget.style.transform = 'scale(1)';
                }}
                title="타이머 설정"
              >
                <img src={timerIcon} alt="Timer" style={{ width: '24px', height: '24px' }} />
              </button>
              <button
                onClick={handleOpenHistory}
                style={{
                  width: '40px',
                  height: '40px',
                  background: 'rgba(0, 0, 0, 0.05)',
                  border: '1px solid rgba(0, 0, 0, 0.1)',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                  padding: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center'
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.08)';
                  e.currentTarget.style.transform = 'scale(1.05)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.05)';
                  e.currentTarget.style.transform = 'scale(1)';
                }}
                title="히스토리"
              >
                <img src={historyIcon} alt="History" style={{ width: '24px', height: '24px' }} />
              </button>
              <button
                onClick={() => setShowRoiModal(true)}
                style={{
                  width: '40px',
                  height: '40px',
                  background: 'rgba(0, 0, 0, 0.05)',
                  border: '1px solid rgba(0, 0, 0, 0.1)',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                  padding: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center'
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.08)';
                  e.currentTarget.style.transform = 'scale(1.05)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.05)';
                  e.currentTarget.style.transform = 'scale(1)';
                }}
                title="ROI 설정"
              >
                <img src={roiIcon} alt="ROI" style={{ width: '24px', height: '24px' }} />
              </button>
              <button
                onClick={() => setShowSettings(true)}
                style={{
                  width: '40px',
                  height: '40px',
                  background: 'rgba(0, 0, 0, 0.05)',
                  border: '1px solid rgba(0, 0, 0, 0.1)',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                  padding: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center'
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.08)';
                  e.currentTarget.style.transform = 'scale(1.05)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.05)';
                  e.currentTarget.style.transform = 'scale(1)';
                }}
                title="설정"
              >
                <img src={settingIcon} alt="Settings" style={{ width: '24px', height: '24px' }} />
              </button>
            </div>
          </>
        )}

        {!isSelecting && showSettings && (
          <>
            <button
              onClick={() => setShowSettings(false)}
              style={{
                position: 'absolute',
                top: '56px',
                left: '12px',
                padding: '6px 12px',
                fontSize: '13px',
                background: 'rgba(0, 0, 0, 0.05)',
                color: '#666',
                border: '1px solid rgba(0, 0, 0, 0.1)',
                borderRadius: '6px',
                cursor: 'pointer',
                transition: 'all 0.15s ease',
                fontWeight: '500'
              }}
            >
              ← 뒤로
            </button>
            <Settings />
          </>
        )}
      </main>

      {/* ROI Configuration Modal */}
      <RoiConfigModal
        isOpen={showRoiModal}
        onClose={() => setShowRoiModal(false)}
        onSelectingChange={handleSelectingChange}
      />

      {/* Timer Settings Modal */}
      <TimerSettingsModal
        isOpen={showTimerSettings}
        onClose={() => setShowTimerSettings(false)}
      />
    </div>
  );
}

export default App;
