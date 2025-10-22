import { useState, useRef, useEffect } from 'react';
import { RoiSelector } from './RoiSelector';
import {
  captureRegion,
  bytesToDataUrl,
  maximizeWindowForROI,
  restoreWindow,
  initScreenCapture,
  setAlwaysOnTop,
  type Roi,
} from '../lib/tauri';
import { useRoiStore, type RoiType } from '../stores/roiStore';
import './RoiManager.css';

interface WindowState {
  width: number;
  height: number;
  x: number;
  y: number;
}

interface RoiManagerProps {
  onSelectingChange?: (isSelecting: boolean) => void;
}

interface RoiCardData {
  type: RoiType;
  label: string;
  color: string;
  description: string;
}

const ROI_TYPES: RoiCardData[] = [
  {
    type: 'level',
    label: 'Level',
    color: '#4CAF50',
    description: '캐릭터 레벨 영역',
  },
  {
    type: 'exp',
    label: 'EXP',
    color: '#2196F3',
    description: '경험치 퍼센트 영역',
  },
  {
    type: 'hp',
    label: 'HP',
    color: '#F44336',
    description: '체력 영역',
  },
  {
    type: 'mp',
    label: 'MP',
    color: '#9C27B0',
    description: '마나 영역',
  },
  // {
  //   type: 'meso',
  //   label: 'Meso',
  //   color: '#FF9800',
  //   description: '메소 영역',
  // },
];

export function RoiManager({ onSelectingChange }: RoiManagerProps) {
  const [isSelecting, setIsSelecting] = useState(false);
  const [currentRoiType, setCurrentRoiType] = useState<RoiType | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [previews, setPreviews] = useState<Record<RoiType, string | null>>({
    level: null,
    exp: null,
    hp: null,
    mp: null,
  });
  const windowStateRef = useRef<WindowState | null>(null);

  const { levelRoi, expRoi, hpRoi, mpRoi, setRoi, removeRoi, loadAllRois } = useRoiStore();

  // Initialize screen capture and load saved ROIs
  useEffect(() => {
    const init = async () => {
      await initScreenCapture();
      await loadAllRois();
      setIsInitialized(true);
    };
    init();
  }, [loadAllRois]);

  // Generate previews for configured ROIs
  useEffect(() => {
    const generatePreviews = async () => {
      const rois = { level: levelRoi, exp: expRoi, hp: hpRoi, mp: mpRoi };

      for (const [type, roi] of Object.entries(rois)) {
        if (roi && !previews[type as RoiType]) {
          try {
            const bytes = await captureRegion(roi);
            const dataUrl = bytesToDataUrl(bytes);
            setPreviews((prev) => ({ ...prev, [type]: dataUrl }));
          } catch (err) {
            console.error(`Failed to generate preview for ${type}:`, err);
          }
        }
      }
    };

    if (isInitialized) {
      generatePreviews();
    }
  }, [levelRoi, expRoi, hpRoi, mpRoi, isInitialized]);

  const handleSelectClick = async (type: RoiType) => {
    setCurrentRoiType(type);
    await setAlwaysOnTop(true);
    windowStateRef.current = await maximizeWindowForROI();
    setIsSelecting(true);
    onSelectingChange?.(true);
  };

  const handleRoiSelected = async (roi: Roi) => {
    if (!currentRoiType) return;

    // Save ROI to store
    await setRoi(currentRoiType, roi);

    // Generate preview
    try {
      const bytes = await captureRegion(roi);
      const dataUrl = bytesToDataUrl(bytes);
      setPreviews((prev) => ({ ...prev, [currentRoiType]: dataUrl }));
    } catch (err) {
      console.error('Failed to capture preview:', err);
    }

    // Restore window
    await setAlwaysOnTop(false);
    if (windowStateRef.current) {
      await restoreWindow(windowStateRef.current);
      windowStateRef.current = null;
    }

    await new Promise((resolve) => setTimeout(resolve, 100));
    setIsSelecting(false);
    setCurrentRoiType(null);
    onSelectingChange?.(false);
  };

  const handleCancel = async () => {
    await setAlwaysOnTop(false);
    if (windowStateRef.current) {
      await restoreWindow(windowStateRef.current);
      windowStateRef.current = null;
    }

    await new Promise((resolve) => setTimeout(resolve, 100));
    setIsSelecting(false);
    setCurrentRoiType(null);
    onSelectingChange?.(false);
  };

  const handleRemoveRoi = async (type: RoiType) => {
    await removeRoi(type);
    setPreviews((prev) => ({ ...prev, [type]: null }));
  };

  const getRoi = (type: RoiType): Roi | null => {
    switch (type) {
      case 'level':
        return levelRoi;
      case 'exp':
        return expRoi;
      case 'hp':
        return hpRoi;
      case 'mp':
        return mpRoi;
    }
  };

  if (isSelecting && currentRoiType) {
    return <RoiSelector onRoiSelected={handleRoiSelected} onCancel={handleCancel} />;
  }

  return (
    <div className="roi-manager">
      <div className="roi-header">
        <h2>영역 설정</h2>
        <p className="roi-subtitle">
          게임 화면에서 추적할 영역을 선택하세요
        </p>
      </div>

      <div className="roi-grid">
        {ROI_TYPES.map(({ type, label, color, description }) => {
          const roi = getRoi(type);
          const preview = previews[type];
          const isConfigured = roi !== null;

          return (
            <div key={type} className="roi-card">
              <div className="roi-card-header" style={{ borderLeftColor: color }}>
                <div className="roi-card-title">
                  <span className="roi-label" style={{ color }}>
                    {label}
                  </span>
                  {isConfigured ? (
                    <span className="roi-status configured">✓ 설정됨</span>
                  ) : (
                    <span className="roi-status unconfigured">미설정</span>
                  )}
                </div>
                <p className="roi-description">{description}</p>
              </div>

              {preview ? (
                <div className="roi-preview">
                  <img src={preview} alt={`${label} preview`} />
                </div>
              ) : (
                <div className="roi-preview-empty">
                  <span className="roi-preview-icon">📸</span>
                  <span className="roi-preview-text">미리보기 없음</span>
                </div>
              )}

              {roi && (
                <div className="roi-info">
                  <span className="roi-coordinates">
                    {roi.width} × {roi.height}
                  </span>
                  <span className="roi-position">
                    ({roi.x}, {roi.y})
                  </span>
                </div>
              )}

              <div className="roi-actions">
                <button
                  onClick={() => handleSelectClick(type)}
                  disabled={!isInitialized}
                  className="roi-button select"
                  style={{ borderColor: color }}
                >
                  {isConfigured ? '재선택' : '영역 선택'}
                </button>

                {isConfigured && (
                  <button
                    onClick={() => handleRemoveRoi(type)}
                    className="roi-button remove"
                  >
                    삭제
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {!isInitialized && (
        <div className="roi-loading">
          <div className="spinner"></div>
          <span>화면 캡처 초기화 중...</span>
        </div>
      )}
    </div>
  );
}
