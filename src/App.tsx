import { useState, useCallback, useEffect } from "react";
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { listen } from '@tauri-apps/api/event';
import { CompactRoiManager } from "./components/CompactRoiManager";
import { Settings } from "./components/Settings";
import { TimerSettingsModal } from "./components/TimerSettingsModal";
import { useSettingsStore } from "./stores/settingsStore";
import { useRoiStore } from "./stores/roiStore";
import { useTrackingStore } from "./stores/trackingStore";
import { useSessionStore } from "./stores/sessionStore";
import { useTimerSettingsStore } from "./stores/timerSettingsStore";
import { useMesoStore } from "./stores/mesoStore";
import { useParallelOcrTracker } from "./hooks/useParallelOcrTracker";
import { initScreenCapture, autoDetectRois } from "./lib/tauri";
import { checkOcrHealth } from "./lib/ocrCommands";
import { formatCompact, formatKoreanNumber } from "./lib/expCommands";
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
import mesoIcon from "/icons/meso.png";
import { SessionArchive } from './components/SessionArchive';

function App() {
  const [isSelecting, setIsSelecting] = useState(false);
  const [showRoiModal, setShowRoiModal] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showTimerSettings, setShowTimerSettings] = useState(false);
  const [showMesoModal, setShowMesoModal] = useState(false);
  const [showSessionArchive, setShowSessionArchive] = useState(false);
  const [mesoInputStart, setMesoInputStart] = useState('');
  const [mesoInputEnd, setMesoInputEnd] = useState('');
  const [potionPriceInputHp, setPotionPriceInputHp] = useState('');
  const [potionPriceInputMp, setPotionPriceInputMp] = useState('');
  const [previewMeso, setPreviewMeso] = useState<{start: number | null, end: number | null, hpPrice: number, mpPrice: number} | null>(null);
  
  // Timestamped EXP data for per-interval calculation and history
  const [expDataPoints, setExpDataPoints] = useState<Array<{
    timestamp: number;
    totalExp: number;
    hpPotions: number;
    mpPotions: number;
  }>>([]);
  const [ocrHealthy, setOcrHealthy] = useState(false);
  const [autoDetectCompleted, setAutoDetectCompleted] = useState(false);

  const backgroundOpacity = useSettingsStore((state) => state.backgroundOpacity);
  const targetDuration = useSettingsStore((state) => state.targetDuration);
  const { levelRoi, expRoi } = useRoiStore();
  const {
    state: trackingState,
    elapsedSeconds,
    pausedSeconds,
    sessionStartTime,
    startPercentage,
    startTracking,
    pauseTracking,
    resetTracking,
    incrementTimer,
    setStartPercentage,
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
    setHpPotionPrice,
    setMpPotionPrice,
    resetSession: resetMesoSession,
    calculateMesoGained,
    calculatePotionCost,
    calculateNetProfit,
  } = useMesoStore();

  // Parallel OCR Tracker hook
  const parallelOcrTracker = useParallelOcrTracker();

  // Check if any ROI is configured
  const hasAnyRoi = levelRoi !== null || expRoi !== null;

  // Screen capture initialization state
  const [screenCaptureReady, setScreenCaptureReady] = useState(false);

  // Initialize screen capture on app start
  useEffect(() => {
    const initCapture = async () => {
      try {
        await initScreenCapture();
        console.log('✅ Screen capture initialized successfully');
        setScreenCaptureReady(true);
      } catch (error) {
        console.error('❌ Failed to initialize screen capture:', error);
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

  // Auto-detect ROIs once everything is ready (one-time on app start)
  useEffect(() => {
    // Wait for both screen capture and OCR to be ready
    if (!screenCaptureReady || !ocrHealthy || autoDetectCompleted) return;

    console.log('🚀 All systems ready, starting auto-detect...');

    const performAutoDetect = async () => {
      let attempts = 0;
      let detected = false;

      while (attempts < 3 && !detected) {
        attempts++;
        try {
          console.log(`Auto-detecting ROIs (attempt ${attempts}/3)...`);
          const result = await autoDetectRois();

          // Save detected ROIs
          const { setRoi, setLevelWithBoxes } = useRoiStore.getState();

          if (result.level) {
            // If level boxes are available, use setLevelWithBoxes to store both ROI and boxes
            if (result.level_boxes && result.level_boxes.length > 0) {
              await setLevelWithBoxes(result.level, result.level_boxes);
              console.log(`✅ Level ROI auto-detected with ${result.level_boxes.length} digit boxes`);
            } else {
              await setRoi('level', result.level);
              console.log('✅ Level ROI auto-detected and saved');
            }
          }

          if (result.inventory) {
            await setRoi('inventory', result.inventory);
            console.log('✅ Inventory ROI auto-detected and saved');
          }

          // Consider successful if at least one ROI was detected
          if (result.level || result.inventory) {
            detected = true;
            console.log('✅ Auto-detect completed successfully');
          }
        } catch (err) {
          console.error(`Auto-detect attempt ${attempts} failed:`, err);
        }
      }

      if (!detected) {
        console.warn('⚠️ Failed to auto-detect ROIs after 3 attempts');
      }

      setAutoDetectCompleted(true);
    };

    performAutoDetect();
  }, [screenCaptureReady, ocrHealthy, autoDetectCompleted]);

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

  // Save starting percentage when tracking starts and first stats arrive
  useEffect(() => {
    if (trackingState === 'tracking' && startPercentage === null && parallelOcrTracker.stats?.percentage !== null && parallelOcrTracker.stats?.percentage !== undefined) {
      setStartPercentage(parallelOcrTracker.stats.percentage);
    }
  }, [trackingState, startPercentage, parallelOcrTracker.stats?.percentage, setStartPercentage]);

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
        return { label: `${intervalLabel} (예상)`, value: formatCompact(avgExp) };
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
          return { label: `${intervalLabel} (예상)`, value: formatCompact(avgExp) };
        }
      }
      
      // Fallback to current rate if not enough data points
      const expPerSecond = stats.total_exp / stats.elapsed_seconds;
      const avgExp = Math.floor(expPerSecond * intervalSeconds);
      return { label: `${intervalLabel} (예상)`, value: formatCompact(avgExp) };
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
      
      return { label: intervalLabel, value: formatCompact(expGained) };
    }
    
    // Not enough data points, use current average
    const cappedSeconds = Math.min(stats.elapsed_seconds, intervalSeconds);
    const expPerSecond = stats.total_exp / stats.elapsed_seconds;
    const avgExp = Math.floor(expPerSecond * cappedSeconds);
    return { label: intervalLabel, value: formatCompact(avgExp) };
  };

  const averageData = calculateAverage();


  // Calculate level up ETA
  const calculateLevelUpETA = (): string => {
    const stats = parallelOcrTracker.stats;
    if (!stats || !stats.level || stats.exp_per_hour === 0) {
      return '?시간 ?분';
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
    const currentExp = stats.exp || 0;
    
    // Validate level range
    if (currentLevel < 1 || currentLevel >= 200) {
      return '?시간 ?분';
    }
    
    // Get required exp for next level
    const requiredExp = expTable[currentLevel];
    
    if (!requiredExp) {
      return '?시간 ?분';
    }
    
    // Calculate remaining exp to next level
    const remainingExp = requiredExp - currentExp;
    
    // Calculate hours needed
    const hoursNeeded = remainingExp / stats.exp_per_hour;
    
    if (hoursNeeded < 0 || !isFinite(hoursNeeded)) {
      return '?시간 ?분';
    }
    
    // Format as hours and minutes
    const hours = Math.floor(hoursNeeded);
    const minutes = Math.floor((hoursNeeded - hours) * 60);
    
    if (hours > 999) {
      return '999시간+';
    }
    
    if (hours === 0) {
      return `${minutes}분`;
    }
    
    return `${hours}시간 ${minutes}분`;
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
    
    const window = getCurrentWindow();
    await window.setResizable(false);
    await window.setSize(new LogicalSize(520, 360));
  };

  const handleCloseRoiModal = async () => {
    setShowRoiModal(false);
    
    const window = getCurrentWindow();
    await window.setResizable(true);
    await window.setSize(new LogicalSize(540, 130));
  };

  const handleOpenMesoModal = async () => {
    setMesoInputStart(startMeso !== null ? startMeso.toString() : '');
    setMesoInputEnd(endMeso !== null ? endMeso.toString() : '');
    setPotionPriceInputHp(hpPotionPrice === 0 ? '' : hpPotionPrice.toString());
    setPotionPriceInputMp(mpPotionPrice === 0 ? '' : mpPotionPrice.toString());
    setPreviewMeso(null);
    setShowMesoModal(true);
    
    const window = getCurrentWindow();
    await window.setResizable(false);
    await window.setSize(new LogicalSize(560, 630));
  };

  const handleCloseMesoModal = async () => {
    setShowMesoModal(false);
    setPreviewMeso(null);
    
    const window = getCurrentWindow();
    await window.setResizable(true);
    await window.setSize(new LogicalSize(540, 130));
  };

  const handleOpenSessionArchive = async () => {
    setShowSessionArchive(true);
    
    const window = getCurrentWindow();
    await window.setResizable(false);
    await window.setSize(new LogicalSize(540, 700));
  };

  const handleCloseSessionArchive = async () => {
    setShowSessionArchive(false);
    
    const window = getCurrentWindow();
    await window.setResizable(true);
    await window.setSize(new LogicalSize(540, 130));
  };

  const handleMesoCalculate = () => {
    let startValue: number | null = null;
    let endValue: number | null = null;
    let hpPrice = 0;
    let mpPrice = 0;

    if (mesoInputStart.trim() !== '') {
      const parsed = parseInt(mesoInputStart.replace(/,/g, ''));
      if (!isNaN(parsed) && parsed >= 0) {
        startValue = parsed;
      }
    }

    if (mesoInputEnd.trim() !== '') {
      const parsed = parseInt(mesoInputEnd.replace(/,/g, ''));
      if (!isNaN(parsed) && parsed >= 0) {
        endValue = parsed;
      }
    }

    if (potionPriceInputHp.trim() !== '') {
      const parsed = parseInt(potionPriceInputHp.replace(/,/g, ''));
      if (!isNaN(parsed) && parsed >= 0) {
        hpPrice = parsed;
      }
    }
    
    if (potionPriceInputMp.trim() !== '') {
      const parsed = parseInt(potionPriceInputMp.replace(/,/g, ''));
      if (!isNaN(parsed) && parsed >= 0) {
        mpPrice = parsed;
      }
    }

    setPreviewMeso({ start: startValue, end: endValue, hpPrice, mpPrice });
  };

  const handleMesoSubmit = async () => {
    if (mesoInputStart.trim() !== '') {
      const startValue = parseInt(mesoInputStart.replace(/,/g, ''));
      if (!isNaN(startValue) && startValue >= 0) {
        setStartMeso(startValue);
      }
    } else {
      setStartMeso(null);
    }

    if (mesoInputEnd.trim() !== '') {
      const endValue = parseInt(mesoInputEnd.replace(/,/g, ''));
      if (!isNaN(endValue) && endValue >= 0) {
        setEndMeso(endValue);
      }
    } else {
      setEndMeso(null);
    }

    if (potionPriceInputHp.trim() !== '') {
      const hpPrice = parseInt(potionPriceInputHp.replace(/,/g, ''));
      if (!isNaN(hpPrice) && hpPrice >= 0) {
        setHpPotionPrice(hpPrice);
      }
    }
    
    if (potionPriceInputMp.trim() !== '') {
      const mpPrice = parseInt(potionPriceInputMp.replace(/,/g, ''));
      if (!isNaN(mpPrice) && mpPrice >= 0) {
        setMpPotionPrice(mpPrice);
      }
    }

    await handleCloseMesoModal();
  };

  const handleToggleTracking = async () => {
    if (!hasAnyRoi) {
      await handleOpenRoiModal();
      return;
    }

    if (trackingState === 'idle') {
      startSession();
      startTracking();
      await parallelOcrTracker.start();
    } else if (trackingState === 'paused') {
      startTracking();
      await parallelOcrTracker.start();
    } else if (trackingState === 'tracking') {
      pauseTracking();
      parallelOcrTracker.stop();
    }
  };

  const handleReset = async () => {
    if (trackingState !== 'idle') {
      endSession();
    }
    resetTracking();
    await parallelOcrTracker.reset();
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

  const handleOpenSettings = async () => {
    setShowSettings(true);
    
    const window = getCurrentWindow();
    await window.setSize(new LogicalSize(480, 500));
  };

  const handleCloseSettings = async () => {
    setShowSettings(false);
    
    const window = getCurrentWindow();
    await window.setSize(new LogicalSize(540, 130));
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
      }}
    >
      <div
        style={{
          width: '100%',
          height: '100%',
          background: isSelecting ? 'transparent' : 'rgba(255, 255, 255, 0.98)',
          borderRadius: 'inherit',
          display: 'flex',
          flexDirection: 'column',
          position: 'relative',
          cursor: (!isSelecting && !showSettings && !showMesoModal && !showRoiModal && !showSessionArchive) ? 'move' : 'default',
          pointerEvents: isSelecting ? 'none' : 'auto',
        }}
        onMouseDown={!isSelecting && !showSettings && !showMesoModal && !showRoiModal && !showSessionArchive ? handleDragStart : undefined}
      >
        {/* OCR Status - Left side of title bar */}
        {!isSelecting && !showSettings && !showMesoModal && !showRoiModal && !showSessionArchive && (
          <div
            onMouseDown={(e) => e.stopPropagation()}
            style={{
              position: 'absolute',
              top: '3px',
              left: '12px',
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
              zIndex: 1000,
              cursor: 'default',
              padding: '1px 4px',
              background: 'rgba(0, 0, 0, 0.03)',
              borderRadius: '4px',
            }}
            title={ocrHealthy ? 'OCR 서버 연결됨' : 'OCR 서버 연결 끊김'}
          >
            <span style={{ fontSize: '6px' }}>
              {ocrHealthy ? '🟢' : '🔴'}
            </span>
            <span style={{ fontSize: '9px', color: '#999', fontWeight: '600' }}>
              OCR
            </span>
          </div>
        )}

        {/* Window Controls are now positioned relative to this container */}
        {!isSelecting && !showSettings && !showMesoModal && !showRoiModal && !showSessionArchive && (
          <div
            onMouseDown={(e) => e.stopPropagation()}
            style={{
              position: 'absolute',
              top: '4px',
              right: '8px',
              display: 'flex',
              gap: '4px',
              zIndex: 1000,
              cursor: 'default',
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
              onMouseEnter={(e) => { e.currentTarget.style.background = 'rgba(0, 0, 0, 0.5)'; }}
              onMouseLeave={(e) => { e.currentTarget.style.background = 'rgba(0, 0, 0, 0.3)'; }}
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
              onMouseEnter={(e) => { e.currentTarget.style.background = '#ff3b30'; }}
              onMouseLeave={(e) => { e.currentTarget.style.background = 'rgba(255, 59, 48, 0.8)'; }}
              title="Close"
            >
              ×
            </button>
          </div>
        )}

        {/* Main Content Area */}
        <main
          style={{
            width: '100%',
            height: '100%',
            boxSizing: 'border-box',
            paddingTop: '32px', // Provides space for the absolute-positioned controls
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          <div
            style={{
              flex: 1,
              display: 'flex',
              flexDirection: (showSettings || showMesoModal || showRoiModal || showSessionArchive) ? 'column' : 'row',
              alignItems: (showSettings || showMesoModal || showRoiModal || showSessionArchive) ? 'stretch' : 'center',
              padding: isSelecting ? '0' : (showSettings || showMesoModal || showRoiModal || showSessionArchive) ? '0' : '0 12px 8px 12px',
              gap: '4px',
              userSelect: (showSettings || showMesoModal || showRoiModal || showSessionArchive) ? 'auto' : 'none',
              overflow: (showSettings || showMesoModal || showRoiModal || showSessionArchive) ? 'auto' : 'hidden',
            }}
          >
            {!isSelecting && !showSettings && !showMesoModal && !showRoiModal && !showSessionArchive && (
              <>
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
                    borderRight: '1px solid rgba(0, 0, 0, 0.1)',
                    cursor: 'default',
                  }}
                >
                  {/* Control Buttons */}
                  <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    marginBottom: '4px'
                  }}>
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
                    {targetDuration > 0 && trackingState === 'tracking' && sessionStartTime ? (
                      (() => {
                        // Calculate target completion time based on session start time
                        const targetTime = new Date(sessionStartTime + targetDuration * 60 * 1000);
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
                      '전투 시간'
                    )}
                  </div>
                </div>

                {/* Section 2: 경험치 정보 */}
                <div style={{
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'center',
                  gap: '4px',
                  paddingLeft: '12px',
                  paddingRight: '12px',
                  borderRight: '1px solid rgba(0, 0, 0, 0.1)',
                  cursor: 'default',
                }}>
                  <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '20px'
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
                        fontSize: '16px',
                        fontWeight: '700',
                        color: '#d32f2f',
                        minWidth: '80px'
                      }}>
                        {levelUpETA}
                      </div>
                    </div>
                    <div>
                      <div style={{ 
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px'
                      }}>
                        <span style={{
                          fontSize: '14px',
                          fontWeight: 600,
                          color: '#666'
                        }}>
                          획득 경험치
                        </span>
                        <button
                          onClick={handleOpenSessionArchive}
                          onMouseDown={(e) => e.stopPropagation()}
                          style={{
                            width: '24px',
                            height: '24px',
                            background: 'rgba(255, 193, 7, 0.1)',
                            border: '1px solid rgba(255, 193, 7, 0.3)',
                            borderRadius: '6px',
                            cursor: 'pointer',
                            padding: 0,
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            fontSize: '14px',
                            transition: 'all 0.15s ease'
                          }}
                          onMouseEnter={(e) => {
                            e.currentTarget.style.transform = 'scale(1.2)';
                          }}
                          onMouseLeave={(e) => {
                            e.currentTarget.style.transform = 'scale(1)';
                          }}
                          title="전투 기록 보관함"
                        >
                          🏆️
                        </button>
                      </div>
                      <div style={{
                        fontSize: '16px',
                        fontWeight: '700',
                        color: '#2196F3',
                        minWidth: '100px'
                      }}>
                        {formatKoreanNumber(parallelOcrTracker.stats?.total_exp || 0)}
                      </div>
                    </div>
                  </div>
                  <div style={{
                    fontSize: '11px',
                    color: '#666',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '0',
                    lineHeight: '1.2'
                  }}>
                    <div>
                      현재: Lv.{parallelOcrTracker.stats?.level || '?'} (
                      {startPercentage !== null && parallelOcrTracker.stats?.percentage !== null ? (
                        <>
                          {startPercentage.toFixed(2)}% → {parallelOcrTracker.stats.percentage!.toFixed(2)}%
                        </>
                      ) : (
                        `${parallelOcrTracker.stats?.percentage?.toFixed(2) || '0.00'}%`
                      )}
                      {startPercentage !== null && parallelOcrTracker.stats?.percentage !== null && (
                        <> | {(parallelOcrTracker.stats.percentage! - startPercentage).toFixed(2)}%</>
                      )}
                      )
                    </div>
                    <div>평균: {formatKoreanNumber(Math.floor((parallelOcrTracker.stats?.exp_per_hour || 0) / 3600))} (1초당)</div>
                  </div>
                </div>

                {/* Section 3: 포션 사용 */}
                <div style={{
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'center',
                  gap: '2px',
                  paddingLeft: '12px',
                  minWidth: '80px',
                  cursor: 'default',
                }}>
                  <div style={{ 
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                    marginBottom: '2px'
                  }}>
                    <span style={{
                      fontWeight: 600, 
                      color: '#666', 
                      fontSize: '14px',
                    }}>
                      포션 사용
                    </span>
                    <button
                      onClick={handleOpenMesoModal}
                      onMouseDown={(e) => e.stopPropagation()}
                      style={{
                        width: '24px',
                        height: '24px',
                        background: 'rgba(255, 193, 7, 0.1)',
                        border: '1px solid rgba(255, 193, 7, 0.3)',
                        borderRadius: '6px',
                        cursor: 'pointer',
                        transition: 'all 0.15s ease',
                        padding: 0,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center'
                      }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.transform = 'scale(1.2)';
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.transform = 'scale(1)';
                      }}
                      title="메소 관리"
                    >
                      <img src={mesoIcon} alt="Meso" style={{ width: '16px', height: '16px' }} />
                    </button>
                  </div>
                  <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '4px'
                  }}>
                    <img src={hpIcon} alt="HP" style={{ width: '18px', height: '18px' }} />
                    <div style={{
                      fontSize: '16px',
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
                    <img src={mpIcon} alt="MP" style={{ width: '18px', height: '18px' }} />
                    <div style={{
                      fontSize: '16px',
                      fontWeight: '700',
                      color: '#2196F3'
                    }}>
                      {parallelOcrTracker.stats?.mp_potions_used || 0}
                    </div>
                  </div>
                </div>
              </>
            )}

            {showRoiModal && (
              <>
                {!isSelecting && (
                  <>
                    {/* Draggable Title Bar for ROI */}
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
                        영역 설정
                      </span>
                    </div>

                    {/* Back Button */}
                    <button
                      onClick={handleCloseRoiModal}
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
                  </>
                )}

                {/* ROI Content with Top Padding */}
                <div style={{ paddingTop: isSelecting ? '0' : '10px', width: '100%', height: '100%', visibility: isSelecting ? 'hidden' : 'visible' }}>
                  <CompactRoiManager onSelectingChange={handleSelectingChange} />
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

                {/* Settings Content with Top Padding */}
                <div style={{ paddingTop: '40px', width: '100%', height: '100%' }}>
                <Settings />
                </div>
              </>
            )}

            {!isSelecting && showMesoModal && (
              <>
                {/* Draggable Title Bar for Meso Management */}
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
                    메소 관리
                  </span>
                </div>

                {/* Back Button */}
                <button
                  onClick={handleCloseMesoModal}
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

                {/* Meso Management Content with Top Padding */}
                <div style={{ paddingTop: '50px', padding: '50px 35px 25px 35px', width: '100%', display: 'flex', flexDirection: 'column', gap: '16px', boxSizing: 'border-box' }}>
                  <p style={{ margin: '0', fontSize: '13px', color: '#666', lineHeight: '1.5' }}>
                    사냥 전후 메소와 포션 단가를 입력하세요.
                  </p>

                  {/* 2x2 Grid Input Section */}
                  <div style={{
                    display: 'grid',
                    gridTemplateColumns: '1fr 1fr',
                    gap: '12px'
                  }}>
                    {/* 시작 메소 */}
                    <div>
                      <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '6px', fontWeight: '600' }}>
                        💰 시작 메소
                      </label>
                      <input
                        type="text"
                        value={mesoInputStart && !isNaN(parseInt(mesoInputStart)) ? parseInt(mesoInputStart).toLocaleString('ko-KR') : mesoInputStart}
                        onChange={(e) => {
                          const value = e.target.value.replace(/,/g, '');
                          if (value === '' || /^\d+$/.test(value)) {
                            setMesoInputStart(value);
                          }
                        }}
                        placeholder="시작 메소"
                        style={{
                          width: '100%',
                          padding: '8px',
                          fontSize: '13px',
                          border: '2px solid #e0e0e0',
                          borderRadius: '6px',
                          boxSizing: 'border-box',
                          outline: 'none',
                          transition: 'border-color 0.15s ease',
                        }}
                        onFocus={(e) => {
                          e.currentTarget.style.borderColor = '#FFC107';
                        }}
                        onBlur={(e) => {
                          e.currentTarget.style.borderColor = '#e0e0e0';
                        }}
                      />
                    </div>

                    {/* 종료 메소 */}
                    <div>
                      <label style={{ display: 'block', fontSize: '12px', color: '#666', marginBottom: '6px', fontWeight: '600' }}>
                        💰 종료 메소
                      </label>
                      <input
                        type="text"
                        value={mesoInputEnd && !isNaN(parseInt(mesoInputEnd)) ? parseInt(mesoInputEnd).toLocaleString('ko-KR') : mesoInputEnd}
                        onChange={(e) => {
                          const value = e.target.value.replace(/,/g, '');
                          if (value === '' || /^\d+$/.test(value)) {
                            setMesoInputEnd(value);
                          }
                        }}
                        placeholder="종료 메소"
                        style={{
                          width: '100%',
                          padding: '8px',
                          fontSize: '13px',
                          border: '2px solid #e0e0e0',
                          borderRadius: '6px',
                          boxSizing: 'border-box',
                          outline: 'none',
                          transition: 'border-color 0.15s ease',
                        }}
                        onFocus={(e) => {
                          e.currentTarget.style.borderColor = '#FFC107';
                        }}
                        onBlur={(e) => {
                          e.currentTarget.style.borderColor = '#e0e0e0';
                        }}
                      />
                    </div>

                    {/* HP 포션 단가 */}
                    <div>
                      <label style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '12px', color: '#666', marginBottom: '6px', fontWeight: '600' }}>
                        <img src={hpIcon} alt="HP" style={{ width: '16px', height: '16px' }} />
                        HP 포션 단가
                      </label>
                      <input
                        type="text"
                        value={potionPriceInputHp && !isNaN(parseInt(potionPriceInputHp)) ? parseInt(potionPriceInputHp).toLocaleString('ko-KR') : potionPriceInputHp}
                        onChange={(e) => {
                          const value = e.target.value.replace(/,/g, '');
                          if (value === '' || /^\d+$/.test(value)) {
                            setPotionPriceInputHp(value);
                          }
                        }}
                        placeholder="HP 포션 단가"
                        style={{
                          width: '100%',
                          padding: '8px',
                          fontSize: '13px',
                          border: '2px solid #e0e0e0',
                          borderRadius: '6px',
                          boxSizing: 'border-box',
                          outline: 'none',
                          transition: 'border-color 0.15s ease',
                        }}
                        onFocus={(e) => {
                          e.currentTarget.style.borderColor = '#f44336';
                        }}
                        onBlur={(e) => {
                          e.currentTarget.style.borderColor = '#e0e0e0';
                        }}
                      />
                    </div>

                    {/* MP 포션 단가 */}
                    <div>
                      <label style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '12px', color: '#666', marginBottom: '6px', fontWeight: '600' }}>
                        <img src={mpIcon} alt="MP" style={{ width: '16px', height: '16px' }} />
                        MP 포션 단가
                      </label>
                      <input
                        type="text"
                        value={potionPriceInputMp && !isNaN(parseInt(potionPriceInputMp)) ? parseInt(potionPriceInputMp).toLocaleString('ko-KR') : potionPriceInputMp}
                        onChange={(e) => {
                          const value = e.target.value.replace(/,/g, '');
                          if (value === '' || /^\d+$/.test(value)) {
                            setPotionPriceInputMp(value);
                          }
                        }}
                        placeholder="MP 포션 단가"
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            handleMesoSubmit();
                          }
                        }}
                        style={{
                          width: '100%',
                          padding: '8px',
                          fontSize: '13px',
                          border: '2px solid #e0e0e0',
                          borderRadius: '6px',
                          boxSizing: 'border-box',
                          outline: 'none',
                          transition: 'border-color 0.15s ease',
                        }}
                        onFocus={(e) => {
                          e.currentTarget.style.borderColor = '#2196F3';
                        }}
                        onBlur={(e) => {
                          e.currentTarget.style.borderColor = '#e0e0e0';
                        }}
                      />
                    </div>
                  </div>

                  {/* Calculate Button */}
                  <div style={{ display: 'flex', justifyContent: 'center' }}>
                    <button
                      onClick={handleMesoCalculate}
                      style={{
                        padding: '10px 24px',
                        fontSize: '13px',
                        fontWeight: '600',
                        background: 'linear-gradient(135deg, #2196F3 0%, #1976D2 100%)',
                        color: 'white',
                        border: 'none',
                        borderRadius: '6px',
                        cursor: 'pointer',
                        transition: 'all 0.15s ease',
                        boxShadow: '0 2px 6px rgba(0, 0, 0, 0.15)',
                        width: '100%'
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
                      계산
                    </button>
                  </div>

                  {/* Result Section */}
                  {(() => {
                    const usePreview = previewMeso !== null;
                    const currentStartMeso = usePreview ? previewMeso.start : startMeso;
                    const currentEndMeso = usePreview ? previewMeso.end : endMeso;
                    const currentHpPrice = usePreview ? previewMeso.hpPrice : hpPotionPrice;
                    const currentMpPrice = usePreview ? previewMeso.mpPrice : mpPotionPrice;
                    
                    const hasPotionPrice = currentHpPrice > 0 || currentMpPrice > 0;
                    const hasMesoData = currentStartMeso !== null && currentEndMeso !== null;
                    const hpUsed = parallelOcrTracker.stats?.hp_potions_used || 0;
                    const mpUsed = parallelOcrTracker.stats?.mp_potions_used || 0;
                    
                    let mesoGained = 0;
                    if (hasMesoData && currentEndMeso !== null && currentStartMeso !== null) {
                      mesoGained = currentEndMeso - currentStartMeso;
                    }
                    
                    const potionCost = (hpUsed * currentHpPrice) + (mpUsed * currentMpPrice);
                    const netProfit = mesoGained - potionCost;

                    if (hasMesoData || hasPotionPrice) {
                      return (
                        <div style={{
                          background: 'rgba(33, 150, 243, 0.05)',
                          border: '1px solid rgba(33, 150, 243, 0.2)',
                          borderRadius: '8px',
                          padding: '12px'
                        }}>
                          <h3 style={{ margin: '0 0 8px 0', fontSize: '13px', fontWeight: '700', color: '#666' }}>
                            📊 수익 계산 {usePreview && <span style={{ fontSize: '11px', color: '#2196F3' }}>(미리보기)</span>}
                          </h3>
                          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', fontSize: '12px' }}>
                            {hasMesoData && (
                              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                <span style={{ color: '#666' }}>메소 획득</span>
                                <span style={{ fontWeight: '700', color: '#4CAF50' }}>
                                  +{mesoGained.toLocaleString('ko-KR')}
                                </span>
                              </div>
                            )}
                            {hasPotionPrice && (
                              <>
                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                  <span style={{ color: '#666' }}>포션 비용</span>
                                  <span style={{ fontWeight: '700', color: '#FF9800' }}>
                                    -{potionCost.toLocaleString('ko-KR')}
                                  </span>
                                </div>
                                <div style={{ fontSize: '11px', color: '#999', marginLeft: '8px' }}>
                                  HP {hpUsed}개 × {currentHpPrice.toLocaleString('ko-KR')} + MP {mpUsed}개 × {currentMpPrice.toLocaleString('ko-KR')}
                                </div>
                              </>
                            )}
                            {hasMesoData && (
                              <>
                                <div style={{ height: '1px', background: 'rgba(0, 0, 0, 0.1)', margin: '2px 0' }} />
                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                  <span style={{ fontWeight: '700', color: '#666' }}>순이익</span>
                                  <span style={{ fontSize: '14px', fontWeight: '700', color: netProfit >= 0 ? '#4CAF50' : '#f44336' }}>
                                    {netProfit.toLocaleString('ko-KR')}
                                  </span>
                                </div>
                              </>
                            )}
                          </div>
                        </div>
                      );
                    }
                    return null;
                  })()}

                  <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end', marginTop: '8px' }}>
                    <button
                      onClick={handleCloseMesoModal}
                      style={{
                        padding: '8px 20px',
                        fontSize: '13px',
                        fontWeight: '600',
                        background: '#f5f5f5',
                        color: '#666',
                        border: 'none',
                        borderRadius: '6px',
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
                      취소
                    </button>
                    <button
                      onClick={handleMesoSubmit}
                      style={{
                        padding: '8px 20px',
                        fontSize: '13px',
                        fontWeight: '600',
                        background: 'linear-gradient(135deg, #4CAF50 0%, #45a049 100%)',
                        color: 'white',
                        border: 'none',
                        borderRadius: '6px',
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
                      💾 저장
                    </button>
                  </div>
                </div>
              </>
            )}

            {!isSelecting && showSessionArchive && (
              <>
                {/* Draggable Title Bar for Session Archive */}
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
                    전투 기록
                  </span>
                </div>

                {/* Back Button */}
                <button
                  onClick={handleCloseSessionArchive}
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

                {/* Session Archive Content with Top Padding */}
                <div style={{ paddingTop: '40px', width: '100%', height: '100%', overflow: 'auto' }}>
                  <SessionArchive 
                    currentSession={parallelOcrTracker.stats ? {
                      elapsed_seconds: elapsedSeconds,
                      total_exp: parallelOcrTracker.stats.total_exp,
                      level: parallelOcrTracker.stats.level || 0,
                      exp_per_second: (parallelOcrTracker.stats.exp_per_hour || 0) / 3600,
                      hp_potions_used: parallelOcrTracker.stats.hp_potions_used,
                      mp_potions_used: parallelOcrTracker.stats.mp_potions_used,
                    } : null}
                  />
                </div>
              </>
            )}
          </div>
        </main>
      </div>
      {/* Timer Settings Modal */}
      <TimerSettingsModal
        isOpen={showTimerSettings}
        onClose={() => setShowTimerSettings(false)}
      />

    </div>
  );
}

export default App;
