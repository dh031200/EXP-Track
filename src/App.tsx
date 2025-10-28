import { useState, useCallback, useEffect } from "react";
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { listen } from '@tauri-apps/api/event';
import { RoiConfigModal } from "./components/RoiConfigModal";
import { Settings } from "./components/Settings";
import { TimerSettingsModal } from "./components/TimerSettingsModal";
import { useSettingsStore } from "./stores/settingsStore";
import { useRoiStore } from "./stores/roiStore";
import { useTrackingStore } from "./stores/trackingStore";
import { useSessionStore } from "./stores/sessionStore";
import { useTimerSettingsStore } from "./stores/timerSettingsStore";
import { useMesoStore } from "./stores/mesoStore";
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
import hpIcon from "/icons/hp.png";
import mpIcon from "/icons/mp.png";

function App() {
  const [isSelecting, setIsSelecting] = useState(false);
  const [showRoiModal, setShowRoiModal] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showTimerSettings, setShowTimerSettings] = useState(false);
  const [showMesoInputModal, setShowMesoInputModal] = useState(false);
  const [mesoInputType, setMesoInputType] = useState<'start' | 'end'>('start');
  const [mesoInputValue, setMesoInputValue] = useState('');
  
  // Timestamped EXP data for per-interval calculation and history
  const [expDataPoints, setExpDataPoints] = useState<Array<{
    timestamp: number;
    totalExp: number;
    hpPotions: number;
    mpPotions: number;
  }>>([]);
  const [ocrHealthy, setOcrHealthy] = useState(false);

  const backgroundOpacity = useSettingsStore((state) => state.backgroundOpacity);
  const targetDuration = useSettingsStore((state) => state.targetDuration);
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

  const {
    startMeso,
    endMeso,
    hpPotionPrice,
    mpPotionPrice,
    setStartMeso,
    setEndMeso,
    resetSession: resetMesoSession,
    calculateMesoGained,
    calculatePotionCost,
    calculateNetProfit,
  } = useMesoStore();

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

  // Ensure window is always on top
  useEffect(() => {
    const ensureAlwaysOnTop = async () => {
      try {
        const window = getCurrentWindow();
        await window.setAlwaysOnTop(true);
        console.log('✅ Window set to always on top');
      } catch (error) {
        console.error('❌ Failed to set always on top:', error);
      }
    };

    ensureAlwaysOnTop();
  }, []);

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

  // Global shortcut: ` (backtick) to toggle tracking
  useEffect(() => {
    const unlisten = listen('global-shortcut-toggle-timer', () => {
      // Don't trigger if user is in settings or selecting ROI
      if (showSettings || isSelecting || showRoiModal) {
        return;
      }

      // Don't trigger if ROI is not set or OCR is not healthy
      if (!hasAnyRoi || !ocrHealthy) {
        console.log('⚠️ Global shortcut: ROI not set or OCR not healthy');
        return;
      }

      console.log('🎹 Global shortcut: Toggling tracking');
      handleToggleTracking();
    });

    return () => {
      unlisten.then(fn => fn());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hasAnyRoi, ocrHealthy, showSettings, isSelecting, showRoiModal, trackingState]);

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


  // Calculate level up ETA
  const calculateLevelUpETA = (): string => {
    const stats = parallelOcrTracker.stats;
    if (!stats || !stats.level || stats.exp_per_hour === 0) {
      return '−';
    }

    // Official Mapleland EXP table (Levels 1-200)
    const expTable: { [key: number]: number } = {
      1: 15, 2: 34, 3: 57, 4: 92, 5: 135, 6: 372, 7: 560, 8: 840, 9: 1242, 10: 1716,
      11: 2360, 12: 3216, 13: 4200, 14: 5460, 15: 7050, 16: 8840, 17: 11040, 18: 13716, 19: 16680, 20: 20216,
      21: 24402, 22: 28980, 23: 34320, 24: 40512, 25: 54900, 26: 57210, 27: 63666, 28: 73080, 29: 83270, 30: 95700,
      31: 108480, 32: 122760, 33: 138666, 34: 155540, 35: 174216, 36: 194832, 37: 216600, 38: 240550, 39: 266682, 40: 294216,
      41: 324240, 42: 356916, 43: 391160, 44: 428280, 45: 468450, 46: 510420, 47: 555680, 48: 604416, 49: 655200, 50: 709716,
      51: 748608, 52: 789631, 53: 832902, 54: 878545, 55: 926689, 56: 977471, 57: 1031036, 58: 1087536, 59: 1147132, 60: 1209904,
      61: 1276301, 62: 1346242, 63: 1420016, 64: 1497832, 65: 1579913, 66: 1666492, 67: 1757185, 68: 1854143, 69: 1955750, 70: 2062925,
      71: 2175973, 72: 2295216, 73: 2420993, 74: 2553663, 75: 2693603, 76: 2841212, 77: 2996910, 78: 3161140, 79: 3334370, 80: 3517903,
      81: 3709827, 82: 3913127, 83: 4127556, 84: 4353756, 85: 4592341, 86: 4844001, 87: 5109452, 88: 5389449, 89: 5684790, 90: 5996316,
      91: 6324914, 92: 6617519, 93: 7037118, 94: 7422752, 95: 7829518, 96: 8258575, 97: 8711144, 98: 9188514, 99: 9620440, 100: 10223168,
      101: 10783397, 102: 11374327, 103: 11997640, 104: 12655110, 105: 13348610, 106: 14080113, 107: 14851703, 108: 15665576, 109: 16524049, 110: 17429566,
      111: 18384706, 112: 19392187, 113: 20454878, 114: 21575805, 115: 22758159, 116: 24005306, 117: 25320796, 118: 26708375, 119: 28171993, 120: 29715818,
      121: 31344244, 122: 33061908, 123: 34873700, 124: 36784778, 125: 38800583, 126: 40926854, 127: 43169645, 128: 45535341, 129: 48030677, 130: 50662758,
      131: 53439077, 132: 56367538, 133: 59456479, 134: 62714694, 135: 66151459, 136: 69776558, 137: 73600313, 138: 77633610, 139: 81887931, 140: 86375389,
      141: 91108760, 142: 96101520, 143: 101367883, 144: 106922842, 145: 112782213, 146: 118962678, 147: 125481832, 148: 132358236, 149: 139611467, 150: 147262175,
      151: 155332142, 152: 163844343, 153: 172823012, 154: 182293713, 155: 192283408, 156: 202820538, 157: 213935103, 158: 225658746, 159: 238024845, 160: 251068606,
      161: 264827165, 162: 279339693, 163: 294647508, 164: 310794191, 165: 327825712, 166: 345790561, 167: 364739883, 168: 384727628, 169: 405810702, 170: 428049128,
      171: 451506220, 172: 476248760, 173: 502347192, 174: 529875818, 175: 558913012, 176: 589541445, 177: 621848316, 178: 655925603, 179: 691870326, 180: 729784819,
      181: 769777027, 182: 811960808, 183: 856456260, 184: 903390063, 185: 952895838, 186: 1005114529, 187: 1060194805, 188: 1118293480, 189: 1179575962, 190: 1244216724,
      191: 1312399800, 192: 1384319309, 193: 1460180007, 194: 1540197871, 195: 1624600714, 196: 1713628833, 197: 1807535693, 198: 1906588648, 199: 2011069705, 200: 2121276324
    };

    const currentLevel = stats.level;
    const currentPercentage = stats.percentage || 0;
    
    // Validate level range
    if (currentLevel < 1 || currentLevel >= 200) {
      return '−';
    }
    
    // Get exp for current and next level
    const currentLevelExp = expTable[currentLevel];
    const nextLevelExp = expTable[currentLevel + 1];
    
    if (!currentLevelExp || !nextLevelExp) {
      return '−';
    }
    
    // Calculate remaining exp to next level
    const expForLevel = nextLevelExp - currentLevelExp;
    const currentExpInLevel = Math.floor(expForLevel * currentPercentage / 100);
    const remainingExp = expForLevel - currentExpInLevel;
    
    // Calculate hours needed
    const hoursNeeded = remainingExp / stats.exp_per_hour;
    
    if (hoursNeeded < 0 || !isFinite(hoursNeeded)) {
      return '−';
    }
    
    // Format as hours and minutes
    const hours = Math.floor(hoursNeeded);
    const minutes = Math.floor((hoursNeeded - hours) * 60);
    
    if (hours > 999) {
      return '999h+';
    }
    
    return `${hours}h ${minutes}m`;
  };

  const levelUpETA = calculateLevelUpETA();

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

  const handleOpenRoiModal = async () => {
    setShowRoiModal(true);
    // Don't force resize - let user keep their preferred size
  };

  const handleCloseRoiModal = async () => {
    setShowRoiModal(false);
    // Don't force resize - let user keep their preferred size
  };

  const handleOpenMesoInput = async (type: 'start' | 'end') => {
    setMesoInputType(type);
    setMesoInputValue('');
    setShowMesoInputModal(true);
    // Don't force resize - let user keep their preferred size
  };

  const handleCloseMesoInput = async () => {
    setShowMesoInputModal(false);
    // Don't force resize - let user keep their preferred size
  };

  const handleToggleTracking = async () => {
    if (!hasAnyRoi) {
      await handleOpenRoiModal();
      return;
    }

    if (trackingState === 'idle') {
      // Show meso input modal for start meso
      await handleOpenMesoInput('start');
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

  const handleStartMesoSubmit = async () => {
    const meso = parseInt(mesoInputValue);
    if (isNaN(meso) || meso < 0) {
      alert('올바른 메소를 입력하세요.');
      return;
    }

    setStartMeso(meso);
    await handleCloseMesoInput();

    // Start new session
    startSession();
    startTracking();
    // Start OCR and exp recording
    await parallelOcrTracker.start();
  };

  const handleSkipMesoInput = async () => {
    await handleCloseMesoInput();
    
    if (mesoInputType === 'start') {
      // Start tracking without meso input
      startSession();
      startTracking();
      await parallelOcrTracker.start();
    }
  };

  const handleEndMesoSubmit = async () => {
    const meso = parseInt(mesoInputValue);
    if (isNaN(meso) || meso < 0) {
      alert('올바른 메소를 입력하세요.');
      return;
    }

    setEndMeso(meso);
    await handleCloseMesoInput();
  };

  const handleReset = async () => {
    if (trackingState !== 'idle') {
      // If tracking is active, ask for end meso
      if (startMeso !== null && endMeso === null) {
        await handleOpenMesoInput('end');
        return;
      }
      // Save session to history before resetting
      endSession();
    }
    resetTracking();
    // Clear exp data and reset to initial state
    await parallelOcrTracker.reset();
    // Reset meso session
    resetMesoSession();
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

  // Handle settings view - resize window
  const handleOpenSettings = async () => {
    setShowSettings(true);
    // Don't force resize - let user keep their preferred size
  };

  const handleCloseSettings = async () => {
    setShowSettings(false);
    // Don't force resize - let user keep their preferred size
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
        borderRadius: isSelecting ? '0' : '10px',
        display: 'flex',
        flexDirection: 'column',
        opacity: isSelecting ? 1 : backgroundOpacity,
        position: 'relative'
      }}
    >
      {/* Main Container with Horizontal Layout */}
      <main 
        onMouseDown={!isSelecting && !showSettings && !showMesoInputModal ? handleDragStart : undefined}
          style={{
          background: isSelecting ? 'transparent' : 'rgba(255, 255, 255, 0.98)',
          height: '100%',
          borderRadius: isSelecting ? '0' : '10px',
          overflow: (showSettings || showMesoInputModal) ? 'auto' : 'hidden',
          boxSizing: 'border-box',
            display: 'flex',
          flexDirection: (showSettings || showMesoInputModal) ? 'column' : 'row',
          alignItems: (showSettings || showMesoInputModal) ? 'stretch' : 'center',
          padding: isSelecting ? '0' : (showSettings || showMesoInputModal) ? '0' : '8px 12px',
          paddingTop: isSelecting ? '0' : (showSettings || showMesoInputModal) ? '0' : '35px',
          paddingBottom: isSelecting ? '0' : (showSettings || showMesoInputModal) ? '0' : '10px',
          gap: '4px',
          position: 'relative',
          cursor: (!isSelecting && !showSettings && !showMesoInputModal) ? 'move' : 'default',
          userSelect: (showSettings || showMesoInputModal) ? 'auto' : 'none'
        }}
      >
        {!isSelecting && !showSettings && !showMesoInputModal && (
          <>
            {/* Window Controls - Top Right */}
          <div
            onMouseDown={(e) => e.stopPropagation()}
            style={{
              position: 'absolute',
              top: '8px',
                right: '8px',
              display: 'flex',
                gap: '4px',
                zIndex: 100
            }}
          >
            <button
              onClick={handleMinimize}
              style={{
                  width: '20px',
                  height: '20px',
                  borderRadius: '4px',
                border: 'none',
                  background: 'rgba(0, 0, 0, 0.3)',
                color: '#fff',
                  fontSize: '14px',
                fontWeight: '300',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                transition: 'all 0.15s ease',
                  paddingBottom: '2px',
              }}
              onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.5)';
              }}
              onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.3)';
              }}
              title="Minimize"
            >
              −
            </button>
            <button
              onClick={handleClose}
              style={{
                  width: '20px',
                  height: '20px',
                  borderRadius: '4px',
                border: 'none',
                background: 'rgba(255, 59, 48, 0.8)',
                color: '#fff',
                  fontSize: '14px',
                fontWeight: '300',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                transition: 'all 0.15s ease',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = '#ff3b30';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'rgba(255, 59, 48, 0.8)';
              }}
              title="Close"
            >
              ×
            </button>
          </div>

            {/* Section 1: 세션 시간 */}
            <div 
              onMouseDown={(e) => e.stopPropagation()}
              style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
                justifyContent: 'center',
                gap: '4px',
                minWidth: '160px',
                paddingRight: '12px',
                borderRight: '1px solid rgba(0, 0, 0, 0.1)'
              }}
            >
              {/* OCR Status + Control Buttons */}
              <div style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                marginBottom: '4px'
              }}>
                <span style={{ fontSize: '10px' }}>{ocrHealthy ? '🟢' : '🔴'}</span>
              <button
                onClick={handleToggleTracking}
                disabled={!hasAnyRoi || !ocrHealthy}
                style={{
                    width: '32px',
                    height: '32px',
                    borderRadius: '8px',
                  border: 'none',
                    background: !hasAnyRoi || !ocrHealthy
                    ? 'rgba(0, 0, 0, 0.1)'
                    : trackingState === 'tracking'
                        ? 'linear-gradient(135deg, #2196F3 0%, #1976D2 100%)'
                      : 'linear-gradient(135deg, #4CAF50 0%, #45a049 100%)',
                    cursor: (hasAnyRoi && ocrHealthy) ? 'pointer' : 'not-allowed',
                  transition: 'all 0.2s ease',
                    boxShadow: (hasAnyRoi && ocrHealthy) ? '0 2px 6px rgba(0, 0, 0, 0.15)' : 'none',
                  padding: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                    opacity: (hasAnyRoi && ocrHealthy) ? 1 : 0.5
                }}
                title={!hasAnyRoi ? 'ROI 설정 필요' : trackingState === 'tracking' ? '일시정지' : '시작'}
              >
                <img
                  src={trackingState === 'tracking' ? pauseIcon : startIcon}
                  alt={trackingState === 'tracking' ? 'Pause' : 'Start'}
                    style={{ width: '20px', height: '20px' }}
                />
              </button>
              <button
                onClick={handleReset}
                disabled={trackingState === 'idle'}
                style={{
                    width: '24px',
                    height: '24px',
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
                title="리셋"
              >
                  <img src={resetIcon} alt="Reset" style={{ width: '14px', height: '14px' }} />
              </button>
              <button
                  onClick={handleOpenRoiModal}
                style={{
                    width: '24px',
                    height: '24px',
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
                  title="ROI 설정"
                >
                  <img src={roiIcon} alt="ROI" style={{ width: '14px', height: '14px' }} />
              </button>
              <button
                  onClick={handleOpenSettings}
                style={{
                    width: '24px',
                    height: '24px',
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
                  title="설정"
              >
                  <img src={settingIcon} alt="Settings" style={{ width: '14px', height: '14px' }} />
              </button>
              </div>

              {/* Timer */}
              <div style={{
                fontSize: '28px',
                fontWeight: 600,
                color: trackingState === 'tracking' ? '#2196F3' : '#666'
              }}>
                {formatTime(elapsedSeconds)}
              </div>
              <div style={{
                fontSize: '12px',
                fontWeight: 600,
                color: '#999',
                textAlign: 'center'
              }}>
                {targetDuration > 0 && trackingState === 'tracking' ? (
                  (() => {
                    const now = new Date();
                    const targetTime = new Date(now.getTime() + (targetDuration * 60 - elapsedSeconds) * 1000);
                    const hours = Math.floor(targetDuration / 60);
                    const minutes = targetDuration % 60;
                    
                    let timeLabel = '';
                    if (hours > 0 && minutes > 0) {
                      timeLabel = `${hours}시간 ${minutes}분`;
                    } else if (hours > 0) {
                      timeLabel = `${hours}시간`;
                    } else {
                      timeLabel = `${minutes}분`;
                    }
                    
                    const targetHours = targetTime.getHours().toString().padStart(2, '0');
                    const targetMinutes = targetTime.getMinutes().toString().padStart(2, '0');
                    const targetSeconds = targetTime.getSeconds().toString().padStart(2, '0');
                    
                    return `${timeLabel} 뒤: ${targetHours}:${targetMinutes}:${targetSeconds}`;
                  })()
                ) : (
                  '세션 시간'
                )}
              </div>
            </div>

            {/* Section 2 & 3: 경험치 + 포션/메소 */}
            <div style={{
              flex: 1,
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'center',
              gap: '8px',
              paddingLeft: '12px'
            }}>
              {/* 첫 번째 줄: 경험치 정보 + 포션 사용 */}
              <div style={{
                display: 'flex',
                alignItems: 'flex-start',
                gap: '12px'
              }}>
                {/* 경험치 정보 */}
                <div style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '40px',
                  flex: 1,
                  paddingRight: '12px',
                  borderRight: '1px solid rgba(0, 0, 0, 0.1)'
                }}>
                  <div>
                    <div style={{
                      fontSize: '14px',
                      fontWeight: 600,
                      color: '#666'
                    }}>
                      레벨업까지
                    </div>
                    <div style={{
                      fontSize: '18px',
                      fontWeight: '700',
                      color: '#d32f2f'
                    }}>
                      {levelUpETA}
                    </div>
                  </div>
                  <div>
                    <div style={{
                      fontSize: '14px',
                      fontWeight: 600,
                      color: '#666'
                    }}>
                      획득 경험치
                    </div>
                    <div style={{
                      fontSize: '16px',
                      fontWeight: '700',
                      color: '#2196F3'
                    }}>
                      {parallelOcrTracker.stats?.total_exp?.toLocaleString('ko-KR') || '0'}
                    </div>
                  </div>
                </div>

                {/* 포션 사용 */}
                <div style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '4px',
                  minWidth: '120px'
                }}>
                  <div style={{ fontWeight: '600', color: '#666', fontSize: '12px' }}>
                    포션 사용
                  </div>
                  <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '12px'
                  }}>
                    <div style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '4px'
                    }}>
                      <img src={hpIcon} alt="HP" style={{ width: '20px', height: '20px' }} />
                      <div style={{
                        fontSize: '14px',
                        fontWeight: '700',
                        color: '#f44336'
                      }}>
                        {parallelOcrTracker.stats?.hp_potions_used || 0}
                      </div>
                    </div>
                    <div style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '4px'
                    }}>
                      <img src={mpIcon} alt="MP" style={{ width: '20px', height: '20px' }} />
                      <div style={{
                        fontSize: '14px',
                        fontWeight: '700',
                        color: '#2196F3'
                      }}>
                        {parallelOcrTracker.stats?.mp_potions_used || 0}
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              {/* 두 번째 줄: 현재 레벨 + 메소 수익 */}
              <div style={{
                display: 'flex',
                alignItems: 'flex-start',
                gap: '12px',
                borderTop: '1px solid rgba(0, 0, 0, 0.05)',
                paddingTop: '6px'
              }}>
                {/* 현재 레벨 정보 */}
                <div style={{
                  fontSize: '11px',
                  color: '#666',
                  flex: 1,
                  paddingRight: '12px',
                  borderRight: '1px solid rgba(0, 0, 0, 0.1)',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '3px'
                }}>
                  <div>현재: Lv.{parallelOcrTracker.stats?.level || '?'} ({parallelOcrTracker.stats?.percentage?.toFixed(2) || '0.00'}%)</div>
                  <div>시간당: {parallelOcrTracker.stats?.exp_per_hour?.toLocaleString('ko-KR') || '0'}</div>
                </div>

                {/* 메소 수익 */}
                <div style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '3px',
                  minWidth: '180px'
                }}>
                  <div style={{ fontWeight: '600', color: '#666', fontSize: '11px' }}>
                    메소 수익
                  </div>
                  {startMeso !== null && endMeso !== null ? (
                    <div style={{ 
                      display: 'flex', 
                      alignItems: 'center',
                      gap: '5px',
                      flexWrap: 'wrap'
                    }}>
                      <span style={{ color: '#FF9800', fontWeight: '600', fontSize: '10px' }}>
                        포션: -{calculatePotionCost(
                          parallelOcrTracker.stats?.hp_potions_used || 0,
                          parallelOcrTracker.stats?.mp_potions_used || 0
                        ).toLocaleString('ko-KR')}
                      </span>
                      <span style={{ color: '#ddd', fontSize: '10px' }}>|</span>
                      <span style={{ 
                        fontWeight: '700',
                        fontSize: '10px',
                        color: calculateNetProfit(
                          parallelOcrTracker.stats?.hp_potions_used || 0,
                          parallelOcrTracker.stats?.mp_potions_used || 0
                        ) >= 0 ? '#4CAF50' : '#f44336'
                      }}>
                        순이익: {calculateNetProfit(
                          parallelOcrTracker.stats?.hp_potions_used || 0,
                          parallelOcrTracker.stats?.mp_potions_used || 0
                        ).toLocaleString('ko-KR')}
                      </span>
                    </div>
                  ) : (
                    <div style={{ color: '#999', fontWeight: '600', fontSize: '10px' }}>
                      메소 정보 미입력
                    </div>
                  )}
                </div>
              </div>
            </div>
          </>
        )}

        {!isSelecting && showSettings && (
          <>
            {/* Draggable Title Bar for Settings */}
            <div
              onMouseDown={handleDragStart}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                right: 0,
                height: '40px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                cursor: 'move',
                zIndex: 999,
                userSelect: 'none'
              }}
            >
              <span style={{
                fontSize: '12px',
                fontWeight: '600',
                color: '#999'
              }}>
                설정
              </span>
            </div>

            {/* Back Button */}
              <button
              onClick={handleCloseSettings}
              onMouseDown={(e) => e.stopPropagation()}
                style={{
                position: 'absolute',
                top: '8px',
                left: '8px',
                padding: '6px 12px',
                fontSize: '13px',
                background: 'rgba(255, 255, 255, 0.95)',
                color: '#333',
                border: '1px solid rgba(0, 0, 0, 0.2)',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                fontWeight: '600',
                zIndex: 1000,
                boxShadow: '0 2px 6px rgba(0, 0, 0, 0.1)'
                }}
                onMouseEnter={(e) => {
                e.currentTarget.style.background = 'rgba(240, 240, 240, 1)';
                  e.currentTarget.style.transform = 'scale(1.05)';
                }}
                onMouseLeave={(e) => {
                e.currentTarget.style.background = 'rgba(255, 255, 255, 0.95)';
                  e.currentTarget.style.transform = 'scale(1)';
                }}
              >
              ← 뒤로
              </button>

            {/* Window Controls */}
            <div
              onMouseDown={(e) => e.stopPropagation()}
              style={{
                position: 'absolute',
                top: '6px',
                right: '8px',
                display: 'flex',
                gap: '4px',
                zIndex: 1000
              }}
            >
              <button
                onClick={handleMinimize}
                style={{
                  width: '20px',
                  height: '20px',
                  borderRadius: '4px',
                  border: 'none',
                  background: 'rgba(0, 0, 0, 0.3)',
                  color: '#fff',
                  fontSize: '14px',
                  fontWeight: '300',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  transition: 'all 0.15s ease',
                  paddingBottom: '2px',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.5)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.3)';
                }}
                title="Minimize"
              >
                −
              </button>
            <button
                onClick={handleClose}
              style={{
                  width: '20px',
                  height: '20px',
                  borderRadius: '4px',
                  border: 'none',
                  background: 'rgba(255, 59, 48, 0.8)',
                  color: '#fff',
                  fontSize: '14px',
                  fontWeight: '300',
                cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                transition: 'all 0.15s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = '#ff3b30';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 59, 48, 0.8)';
                }}
                title="Close"
              >
                ×
            </button>
            </div>

            {/* Settings Content with Top Padding */}
            <div style={{ paddingTop: '40px' }}>
            <Settings />
            </div>
          </>
        )}

        {!isSelecting && showMesoInputModal && (
          <>
            {/* Draggable Title Bar for Meso Input */}
            <div
              onMouseDown={handleDragStart}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                right: 0,
                height: '40px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                cursor: 'move',
                zIndex: 999,
                userSelect: 'none'
              }}
            >
              <span style={{
                fontSize: '12px',
                fontWeight: '600',
                color: '#999'
              }}>
                {mesoInputType === 'start' ? '시작 메소 입력' : '종료 메소 입력'}
              </span>
            </div>

            {/* Back Button */}
            <button
              onClick={handleCloseMesoInput}
              onMouseDown={(e) => e.stopPropagation()}
              style={{
                position: 'absolute',
                top: '8px',
                left: '8px',
                padding: '6px 12px',
                fontSize: '13px',
                background: 'rgba(255, 255, 255, 0.95)',
                color: '#333',
                border: '1px solid rgba(0, 0, 0, 0.2)',
                borderRadius: '6px',
                cursor: 'pointer',
                transition: 'all 0.15s ease',
                fontWeight: '600',
                zIndex: 1000,
                boxShadow: '0 2px 6px rgba(0, 0, 0, 0.1)'
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = 'rgba(240, 240, 240, 1)';
                e.currentTarget.style.transform = 'scale(1.05)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'rgba(255, 255, 255, 0.95)';
                e.currentTarget.style.transform = 'scale(1)';
              }}
            >
              ← 뒤로
            </button>

            {/* Window Controls */}
            <div
              onMouseDown={(e) => e.stopPropagation()}
              style={{
                position: 'absolute',
                top: '6px',
                right: '8px',
                display: 'flex',
                gap: '4px',
                zIndex: 1000
              }}
            >
              <button
                onClick={handleMinimize}
                style={{
                  width: '20px',
                  height: '20px',
                  borderRadius: '4px',
                  border: 'none',
                  background: 'rgba(0, 0, 0, 0.3)',
                  color: '#fff',
                  fontSize: '14px',
                  fontWeight: '300',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  transition: 'all 0.15s ease',
                  paddingBottom: '2px',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.5)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(0, 0, 0, 0.3)';
                }}
                title="Minimize"
              >
                −
              </button>
              <button
                onClick={handleClose}
                style={{
                  width: '20px',
                  height: '20px',
                  borderRadius: '4px',
                  border: 'none',
                  background: 'rgba(255, 59, 48, 0.8)',
                  color: '#fff',
                  fontSize: '14px',
                  fontWeight: '300',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  transition: 'all 0.15s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = '#ff3b30';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 59, 48, 0.8)';
                }}
                title="Close"
              >
                ×
              </button>
            </div>

            {/* Meso Input Content with Top Padding */}
            <div style={{ paddingTop: '50px', padding: '50px 30px 30px 30px' }}>
              <p style={{ margin: '0 0 20px 0', fontSize: '14px', color: '#666', lineHeight: '1.5' }}>
                {mesoInputType === 'start' 
                  ? '사냥을 시작하기 전 현재 보유 중인 총 메소를 입력하세요.'
                  : '사냥이 끝난 후 현재 보유 중인 총 메소를 입력하세요.'}
              </p>
              <input
                type="text"
                value={mesoInputValue ? parseInt(mesoInputValue.replace(/,/g, '')).toLocaleString('ko-KR') : ''}
                onChange={(e) => {
                  const value = e.target.value.replace(/,/g, '');
                  if (value === '' || /^\d+$/.test(value)) {
                    setMesoInputValue(value);
                  }
                }}
                placeholder="메소를 입력하세요"
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    if (mesoInputType === 'start') {
                      handleStartMesoSubmit();
                    } else {
                      handleEndMesoSubmit();
                    }
                  }
                }}
                style={{
                  width: '100%',
                  padding: '14px',
                  fontSize: '16px',
                  border: '2px solid #e0e0e0',
                  borderRadius: '8px',
                  marginBottom: '20px',
                  boxSizing: 'border-box',
                  outline: 'none',
                  transition: 'border-color 0.15s ease',
                }}
                onFocus={(e) => {
                  e.currentTarget.style.borderColor = '#4CAF50';
                }}
                onBlur={(e) => {
                  e.currentTarget.style.borderColor = '#e0e0e0';
                }}
              />
              <div style={{ display: 'flex', gap: '10px', justifyContent: 'space-between' }}>
                <button
                  onClick={handleSkipMesoInput}
                  style={{
                    padding: '12px 24px',
                    fontSize: '14px',
                    fontWeight: '600',
                    background: '#f5f5f5',
                    color: '#999',
                    border: 'none',
                    borderRadius: '8px',
                    cursor: 'pointer',
                    transition: 'all 0.15s ease',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.background = '#e0e0e0';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = '#f5f5f5';
                  }}
                >
                  건너뛰기
                </button>
                <button
                  onClick={mesoInputType === 'start' ? handleStartMesoSubmit : handleEndMesoSubmit}
                  style={{
                    padding: '12px 24px',
                    fontSize: '14px',
                    fontWeight: '600',
                    background: 'linear-gradient(135deg, #4CAF50 0%, #45a049 100%)',
                    color: 'white',
                    border: 'none',
                    borderRadius: '8px',
                    cursor: 'pointer',
                    transition: 'all 0.15s ease',
                    boxShadow: '0 2px 6px rgba(0, 0, 0, 0.15)',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.transform = 'translateY(-1px)';
                    e.currentTarget.style.boxShadow = '0 4px 8px rgba(0, 0, 0, 0.2)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.transform = 'translateY(0)';
                    e.currentTarget.style.boxShadow = '0 2px 6px rgba(0, 0, 0, 0.15)';
                  }}
                >
                  확인
                </button>
              </div>
            </div>
          </>
        )}
      </main>

      {/* ROI Configuration Modal */}
      <RoiConfigModal
        isOpen={showRoiModal}
        onClose={handleCloseRoiModal}
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
