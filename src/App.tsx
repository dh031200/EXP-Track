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
import { useParallelOcrTracker } from "./hooks/useParallelOcrTracker";
import { initScreenCapture } from "./lib/tauri";
import { checkOcrHealth } from "./lib/ocrCommands";
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
  
  // Timestamped EXP data for per-interval calculation and history
  const [expDataPoints, setExpDataPoints] = useState<Array<{
    timestamp: number;
    totalExp: number;
    hpPotions: number;
    mpPotions: number;
  }>>([]);
  const [ocrHealthy, setOcrHealthy] = useState(false);

  const backgroundOpacity = useSettingsStore((state) => state.backgroundOpacity);
  const { levelRoi, expRoi } = useRoiStore();
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

  const { selectedAverageInterval, averageCalculationMode } = useTimerSettingsStore();

  // Parallel OCR Tracker hook
  const parallelOcrTracker = useParallelOcrTracker();

  // Check if any ROI is configured
  const hasAnyRoi = levelRoi !== null || expRoi !== null;

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

  // OCR health check polling - check every 3 seconds until healthy
  useEffect(() => {
    const checkHealth = async () => {
      try {
        const healthy = await checkOcrHealth();
        setOcrHealthy(healthy);
      } catch (error) {
        console.error('OCR health check failed:', error);
        setOcrHealthy(false);
      }
    };

    // Initial check
    checkHealth();

    // Poll every 3 seconds if not healthy
    const interval = setInterval(() => {
      if (!ocrHealthy) {
        checkHealth();
      }
    }, 3000);

    return () => clearInterval(interval);
  }, [ocrHealthy]);

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

  // Record EXP data points every minute for per-interval calculation
  useEffect(() => {
    if (trackingState === 'tracking' && parallelOcrTracker.stats) {
      const interval = setInterval(() => {
        const now = Date.now();
        const totalExp = parallelOcrTracker.stats?.total_exp || 0;
        
        setExpDataPoints(prev => {
          const hpPotions = parallelOcrTracker.stats?.hp_potions_used || 0;
          const mpPotions = parallelOcrTracker.stats?.mp_potions_used || 0;
          const newPoints = [...prev, { timestamp: now, totalExp, hpPotions, mpPotions }];
          // Keep only last 24 hours of data (for history graphs)
          const cutoffTime = now - 24 * 60 * 60 * 1000;
          return newPoints.filter(point => point.timestamp > cutoffTime);
        });
      }, 60000); // Every 1 minute
      
      return () => clearInterval(interval);
    }
  }, [trackingState, parallelOcrTracker.stats]);

  // Reset data points when tracking is reset
  useEffect(() => {
    if (trackingState === 'idle') {
      setExpDataPoints([]);
    }
  }, [trackingState]);

  // Format elapsed seconds as HH:MM:SS
  const formatTime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // Calculate average exp for selected interval
  const calculateAverage = (): { label: string; value: string } | null => {
    if (selectedAverageInterval === 'none' || !parallelOcrTracker.stats) {
      return null;
    }

    const intervalMinutes = {
      'none': 0,
      '1min': 1,
      '5min': 5,
      '10min': 10,
      '30min': 30,
      '1hour': 60,
    }[selectedAverageInterval] || 0;

    const intervalLabel = {
      '1min': '1분',
      '5min': '5분',
      '10min': '10분',
      '30min': '30분',
      '1hour': '시간',
    }[selectedAverageInterval] || '';

    const stats = parallelOcrTracker.stats;
    const intervalSeconds = intervalMinutes * 60;

    // Use real-time stats from OCR tracker
    if (!stats || stats.elapsed_seconds === 0) {
      return { label: intervalLabel, value: '0' };
    }

    // Prediction mode: Use first 1/10 of interval to predict full interval
    // [예상] 경험치: 기준 시간의 1/10 동안 얻은 경험치로 전체 예측
    if (averageCalculationMode === 'prediction') {
      const predictionWindow = intervalSeconds / 10; // 1/10 of interval
      
      if (stats.elapsed_seconds < predictionWindow) {
        // Not enough data yet, show current rate
        const expPerSecond = stats.total_exp / stats.elapsed_seconds;
        const avgExp = Math.floor(expPerSecond * intervalSeconds);
        return { label: `${intervalLabel} (예상)`, value: avgExp.toLocaleString('ko-KR') };
      }
      
      // Use data from prediction window to predict full interval
      const now = Date.now();
      const windowStart = now - (predictionWindow * 1000);
      
      // Find closest data point to window start
      const relevantPoints = expDataPoints.filter(p => p.timestamp >= windowStart);
      
      if (relevantPoints.length >= 2) {
        const firstPoint = relevantPoints[0];
        const lastPoint = relevantPoints[relevantPoints.length - 1];
        const expGained = lastPoint.totalExp - firstPoint.totalExp;
        const timeElapsed = (lastPoint.timestamp - firstPoint.timestamp) / 1000;
        
        if (timeElapsed > 0) {
          const expPerSecond = expGained / timeElapsed;
          const avgExp = Math.floor(expPerSecond * intervalSeconds);
          return { label: `${intervalLabel} (예상)`, value: avgExp.toLocaleString('ko-KR') };
        }
      }
      
      // Fallback to current rate if not enough data points
      const expPerSecond = stats.total_exp / stats.elapsed_seconds;
      const avgExp = Math.floor(expPerSecond * intervalSeconds);
      return { label: `${intervalLabel} (예상)`, value: avgExp.toLocaleString('ko-KR') };
    }

    // Per-interval mode: Actual EXP gained in recent N minutes
    // [분당] 경험치: 최근 N분 동안 실제로 얻은 경험치
    const now = Date.now();
    const windowStart = now - (intervalSeconds * 1000);
    
    // Filter data points within the interval
    const relevantPoints = expDataPoints.filter(p => p.timestamp >= windowStart);
    
    if (relevantPoints.length >= 2) {
      const firstPoint = relevantPoints[0];
      const lastPoint = relevantPoints[relevantPoints.length - 1];
      const expGained = lastPoint.totalExp - firstPoint.totalExp;
      
      return { label: intervalLabel, value: expGained.toLocaleString('ko-KR') };
    }
    
    // Not enough data points, use current average
    const cappedSeconds = Math.min(stats.elapsed_seconds, intervalSeconds);
    const expPerSecond = stats.total_exp / stats.elapsed_seconds;
    const avgExp = Math.floor(expPerSecond * cappedSeconds);
    return { label: intervalLabel, value: avgExp.toLocaleString('ko-KR') };
  };

  const averageData = calculateAverage();

  // Calculate HP/MP potion usage per minute (based on recent interval)
  const calculatePotionUsage = (): { hpPerMinute: number; mpPerMinute: number } => {
    const stats = parallelOcrTracker.stats;
    if (!stats || stats.elapsed_seconds === 0) {
      return { hpPerMinute: 0, mpPerMinute: 0 };
    }

    const intervalMinutes = {
      'none': 0,
      '1min': 1,
      '5min': 5,
      '10min': 10,
      '30min': 30,
      '1hour': 60,
    }[selectedAverageInterval] || 0;
    const intervalSeconds = intervalMinutes * 60;
    const now = Date.now();
    const windowStart = now - (intervalSeconds * 1000);

    // Filter data points within the interval
    const relevantPoints = expDataPoints.filter(p => p.timestamp >= windowStart);

    if (relevantPoints.length >= 2) {
      const firstPoint = relevantPoints[0];
      const lastPoint = relevantPoints[relevantPoints.length - 1];
      const hpUsed = lastPoint.hpPotions - firstPoint.hpPotions;
      const mpUsed = lastPoint.mpPotions - firstPoint.mpPotions;
      const timeElapsed = (lastPoint.timestamp - firstPoint.timestamp) / 1000 / 60; // in minutes

      if (timeElapsed > 0) {
        return {
          hpPerMinute: hpUsed / timeElapsed,
          mpPerMinute: mpUsed / timeElapsed,
        };
      }
    }

    // Not enough data points, use total average
    const elapsedMinutes = stats.elapsed_seconds / 60;
    return {
      hpPerMinute: elapsedMinutes > 0 ? stats.hp_potions_used / elapsedMinutes : 0,
      mpPerMinute: elapsedMinutes > 0 ? stats.mp_potions_used / elapsedMinutes : 0,
    };
  };

  const potionUsage = calculatePotionUsage();

  // Get interval label for display
  const intervalLabel = {
    'none': '시간',
    '1min': '1분',
    '5min': '5분',
    '10min': '10분',
    '30min': '30분',
    '1hour': '시간',
  }[selectedAverageInterval] || '시간';

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
      await parallelOcrTracker.start();
    } else if (trackingState === 'paused') {
      // Resume tracking
      startTracking();
      // Resume OCR and exp recording
      await parallelOcrTracker.start();
    } else if (trackingState === 'tracking') {
      // Pause tracking
      pauseTracking();
      // Pause OCR and exp recording
      parallelOcrTracker.stop();
    }
  };

  const handleReset = async () => {
    if (trackingState !== 'idle') {
      // Save session to history before resetting
      endSession();
    }
    resetTracking();
    // Clear exp data and reset to initial state
    await parallelOcrTracker.reset();
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
      await existingWindow.setFocus();
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
            cursor: 'move',
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
        padding: isSelecting ? '0' : '8px 16px 30px 16px', /* top right bottom left - reduced top padding */
        height: isSelecting ? '100%' : 'calc(100% - 44px)',
        borderBottomLeftRadius: isSelecting ? '0' : '12px',
        borderBottomRightRadius: isSelecting ? '0' : '12px',
        overflow: isSelecting ? 'hidden' : 'hidden',
        boxSizing: 'border-box',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'flex-start', /* Align to top instead of center */
        gap: '2px', /* Minimal gap between sections */
        paddingTop: '20px', /* Add top padding to push content down from titlebar */
        position: 'relative'
      }}>
        {!isSelecting && !showSettings && (
          <>
            {/* OCR Status Indicator - Top Left of main container */}
            {(
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
                  {ocrHealthy ? '🟢' : '🔴'}
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
                disabled={!hasAnyRoi || !ocrHealthy}
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


            {/* EXP Tracker Display - Always visible for better UX */}
            <div style={{
              width: '100%',
              maxWidth: '400px',
              marginTop: '8px' /* Spacing between timer and cards */
            }}>
              <ExpTrackerDisplay
                stats={parallelOcrTracker.stats}
                isTracking={parallelOcrTracker.isRunning()}
                error={null}
                averageData={calculateAverage()}
                calculationMode={averageCalculationMode}
                intervalLabel={intervalLabel}
                potionUsage={potionUsage}
              />
            </div>

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
                disabled={true}
                style={{
                  width: '30px',
                  height: '30px',
                  background: 'rgba(0, 0, 0, 0.03)',
                  border: '1px solid rgba(0, 0, 0, 0.05)',
                  borderRadius: '6px',
                  cursor: 'not-allowed',
                  transition: 'all 0.15s ease',
                  padding: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  opacity: 0.4
                }}
                title="히스토리 (준비 중)"
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
