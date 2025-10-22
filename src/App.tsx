import { useState, useCallback, useEffect } from "react";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { RoiConfigModal } from "./components/RoiConfigModal";
import { Settings } from "./components/Settings";
import { TimerSettingsModal } from "./components/TimerSettingsModal";
import { ExpTrackerDisplay } from "./components/ExpTrackerDisplay";
import { useSettingsStore } from "./stores/settingsStore";
import { useRoiStore } from "./stores/roiStore";
import { useTrackingStore } from "./stores/trackingStore";
import { useSessionStore } from "./stores/sessionStore";
import { useTimerSettingsStore } from "./stores/timerSettingsStore";
import { useExpTracker } from "./hooks/useExpTracker";
import { initScreenCapture } from "./lib/tauri";
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
  } = useTrackingStore();

  const {
    sessions,
    startSession,
    endSession,
    updateSessionDuration,
  } = useSessionStore();

  const { selectedAverageInterval } = useTimerSettingsStore();

  // EXP Tracker hook
  const expTracker = useExpTracker();

  // Check if any ROI is configured
  const hasAnyRoi = levelRoi !== null || expRoi !== null || mapLocationRoi !== null;

  // Initialize screen capture on app start
  useEffect(() => {
    const initCapture = async () => {
      try {
        await initScreenCapture();
        console.log('Screen capture initialized successfully');
      } catch (error) {
        console.error('Failed to initialize screen capture:', error);
      }
    };

    initCapture();
  }, []); // Run only once on mount

  // Timer effect - increment every second when tracking
  useEffect(() => {
    if (trackingState === 'tracking') {
      const interval = setInterval(() => {
        incrementTimer();
        // Update current session duration
        updateSessionDuration(elapsedSeconds + 1, pausedSeconds);
      }, 1000);
      return () => clearInterval(interval);
    }
  }, [trackingState, incrementTimer, elapsedSeconds, pausedSeconds, updateSessionDuration]);

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

  const handleToggleTracking = async () => {
    if (!hasAnyRoi) {
      setShowRoiModal(true);
      return;
    }

    if (trackingState === 'idle') {
      // Start new session
      startSession();
      startTracking();
      // Start OCR and exp recording
      await expTracker.start();
    } else if (trackingState === 'paused') {
      // Resume tracking
      startTracking();
      // Resume OCR and exp recording
      await expTracker.start();
    } else if (trackingState === 'tracking') {
      // Pause tracking
      pauseTracking();
      // Pause OCR and exp recording
      expTracker.stop();
    }
  };

  const handleReset = async () => {
    if (trackingState !== 'idle') {
      // Save session to history before resetting
      endSession();
    }
    resetTracking();
    // Clear exp data and reset to initial state
    await expTracker.reset();
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
        padding: isSelecting ? '0' : '16px 16px 30px 16px', /* top right bottom left */
        height: isSelecting ? '100%' : 'calc(100% - 44px)',
        borderBottomLeftRadius: isSelecting ? '0' : '12px',
        borderBottomRightRadius: isSelecting ? '0' : '12px',
        overflow: isSelecting ? 'hidden' : 'hidden',
        boxSizing: 'border-box',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '12px',
        position: 'relative'
      }}>
        {!isSelecting && !showSettings && (
          <>
            {/* OCR Status Indicator - Top Left of main container */}
            {(expTracker.state.isTracking || expTracker.state.stats) && (
              <div style={{
                position: 'absolute',
                top: '12px',
                left: '12px',
                display: 'flex',
                alignItems: 'center',
                gap: '3px',
                padding: '3px 6px',
                background: 'rgba(0, 0, 0, 0.02)',
                borderRadius: '4px',
                zIndex: 10
              }}>
                <span style={{ fontSize: '10px', lineHeight: 1 }}>
                  {expTracker.state.ocrStatus === 'success' && '🟢'}
                  {expTracker.state.ocrStatus === 'warning' && '🟡'}
                  {expTracker.state.ocrStatus === 'error' && '🔴'}
                </span>
                <span style={{
                  fontSize: '9px',
                  fontWeight: 600,
                  color: '#666',
                  textTransform: 'uppercase',
                  letterSpacing: '0.5px'
                }}>OCR</span>
              </div>
            )}
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

              {/* Timer Display - Compact Size */}
              <div style={{
                fontSize: '32px',
                fontWeight: '700',
                color: trackingState === 'tracking' ? '#4CAF50' : '#666',
                fontFamily: 'monospace',
                letterSpacing: '2px',
                textAlign: 'center'
              }}>
                {formatTime(elapsedSeconds)}
              </div>
            </div>


            {/* EXP Tracker Display - Only show when tracking or has data */}
            {(expTracker.state.isTracking || expTracker.state.stats) && (
              <div style={{
                width: '100%',
                maxWidth: '400px',
                marginTop: '10px' /* Reduced from 16px */
              }}>
                <ExpTrackerDisplay
                  stats={expTracker.state.stats}
                  level={expTracker.state.level}
                  exp={expTracker.state.exp}
                  percentage={expTracker.state.percentage}
                  mapName={expTracker.state.mapName}
                  isTracking={expTracker.state.isTracking}
                  error={expTracker.state.error}
                  ocrStatus={expTracker.state.ocrStatus}
                  averageData={calculateAverage()}
                />
              </div>
            )}

            {/* Average EXP Display removed - now integrated into ExpTrackerDisplay */}

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
                  width: '30px',
                  height: '30px',
                  borderRadius: '6px',
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
                  style={{ width: '20px', height: '20px' }}
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
                  width: '30px',
                  height: '30px',
                  background: 'rgba(0, 0, 0, 0.05)',
                  border: '1px solid rgba(0, 0, 0, 0.1)',
                  borderRadius: '6px',
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
                <img src={timerIcon} alt="Timer" style={{ width: '20px', height: '20px' }} />
              </button>
              <button
                onClick={handleOpenHistory}
                style={{
                  width: '30px',
                  height: '30px',
                  background: 'rgba(0, 0, 0, 0.05)',
                  border: '1px solid rgba(0, 0, 0, 0.1)',
                  borderRadius: '6px',
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
                <img src={historyIcon} alt="History" style={{ width: '20px', height: '20px' }} />
              </button>
              <button
                onClick={() => setShowRoiModal(true)}
                style={{
                  width: '30px',
                  height: '30px',
                  background: 'rgba(0, 0, 0, 0.05)',
                  border: '1px solid rgba(0, 0, 0, 0.1)',
                  borderRadius: '6px',
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
                <img src={roiIcon} alt="ROI" style={{ width: '20px', height: '20px' }} />
              </button>
              <button
                onClick={() => setShowSettings(true)}
                style={{
                  width: '30px',
                  height: '30px',
                  background: 'rgba(0, 0, 0, 0.05)',
                  border: '1px solid rgba(0, 0, 0, 0.1)',
                  borderRadius: '6px',
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
                <img src={settingIcon} alt="Settings" style={{ width: '20px', height: '20px' }} />
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
