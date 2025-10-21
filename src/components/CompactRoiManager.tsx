import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
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
import { invoke } from '@tauri-apps/api/core';
import './CompactRoiManager.css';

interface WindowState {
  width: number;
  height: number;
  x: number;
  y: number;
}

interface CompactRoiManagerProps {
  onSelectingChange?: (isSelecting: boolean) => void;
}

const ROI_CONFIGS = [
  { type: 'level' as RoiType, label: 'Level', icon: '📊', color: '#4CAF50' },
  { type: 'exp' as RoiType, label: 'EXP', icon: '📈', color: '#2196F3' },
  { type: 'mapLocation' as RoiType, label: 'Map', icon: '🗺️', color: '#9C27B0' },
  // { type: 'meso' as RoiType, label: 'Meso', icon: '💰', color: '#FF9800' }, // Commented out temporarily
];

export function CompactRoiManager({ onSelectingChange }: CompactRoiManagerProps) {
  const [isSelecting, setIsSelecting] = useState(false);
  const [currentRoiType, setCurrentRoiType] = useState<RoiType | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const windowStateRef = useRef<WindowState | null>(null);

  const { levelRoi, expRoi, mapLocationRoi, setRoi, removeRoi, loadAllRois } = useRoiStore(); // mesoRoi commented out

  useEffect(() => {
    const init = async () => {
      await initScreenCapture();
      await loadAllRois();
      setIsInitialized(true);
    };
    init();
  }, [loadAllRois]);

  const getRoi = (type: RoiType): Roi | null => {
    switch (type) {
      case 'level': return levelRoi;
      case 'exp': return expRoi;
      case 'mapLocation': return mapLocationRoi;
      // case 'meso': return mesoRoi; // Commented out temporarily
    }
  };

  const handleSelectClick = async (type: RoiType) => {
    setCurrentRoiType(type);
    await setAlwaysOnTop(true);
    windowStateRef.current = await maximizeWindowForROI();
    setIsSelecting(true);
    onSelectingChange?.(true);
  };

  const handleRoiSelected = async (roi: Roi) => {
    if (!currentRoiType) return;

    const roiType = currentRoiType;

    // Save ROI
    await setRoi(roiType, roi);

    // Step 1: Hide overlay and selection UI
    setIsSelecting(false);
    onSelectingChange?.(false);

    // Step 2: Restore window to original size
    await setAlwaysOnTop(false);
    if (windowStateRef.current) {
      await restoreWindow(windowStateRef.current);
      windowStateRef.current = null;
    }

    // Step 3: Wait 500ms for UI to settle
    await new Promise((resolve) => setTimeout(resolve, 500));

    // Step 4: Capture the clean screen
    try {
      const bytes = await captureRegion(roi);
      const dataUrl = bytesToDataUrl(bytes);

      // Save to temp folder via Tauri command
      await invoke('save_roi_preview', {
        roiType: roiType,
        imageData: dataUrl.split(',')[1], // Remove data:image/png;base64, prefix
      });
    } catch (err) {
      console.error('Failed to save preview:', err);
    }

    setCurrentRoiType(null);
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

  const handleViewPreview = async (type: RoiType) => {
    try {
      await invoke('open_roi_preview', { roiType: type });
    } catch (err) {
      console.error('Failed to open preview:', err);
    }
  };

  const handleRemoveRoi = async (type: RoiType) => {
    await removeRoi(type);
  };

  // Render RoiSelector outside modal container using Portal
  const roiSelectorPortal = isSelecting && currentRoiType ? createPortal(
    <RoiSelector onRoiSelected={handleRoiSelected} onCancel={handleCancel} />,
    document.body
  ) : null;

  return (
    <>
      <div className="compact-roi-manager">
        <div className="roi-buttons">
          {ROI_CONFIGS.map(({ type, label, icon, color }) => {
            const roi = getRoi(type);
            const isConfigured = roi !== null;

            return (
              <div key={type} className="roi-button-group">
                <button
                  onClick={() => handleSelectClick(type)}
                  disabled={!isInitialized}
                  className="roi-select-btn"
                  style={{ borderColor: color }}
                  title={`${label} 영역 ${isConfigured ? '재' : ''}선택`}
                >
                  <span className="roi-icon">{icon}</span>
                  <span className="roi-label">{label}</span>
                  {isConfigured && <span className="roi-check">✓</span>}
                </button>

                {isConfigured && (
                  <div className="roi-actions-compact">
                    <button
                      onClick={() => handleViewPreview(type)}
                      className="roi-action-btn view"
                      title="미리보기 열기"
                    >
                      👁️
                    </button>
                    <button
                      onClick={() => handleRemoveRoi(type)}
                      className="roi-action-btn delete"
                      title="삭제"
                    >
                      🗑️
                    </button>
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {!isInitialized && (
          <div className="roi-init-status">
            <span className="spinner-small"></span>
            <span>초기화 중...</span>
          </div>
        )}
      </div>

      {/* Render RoiSelector as Portal to document.body */}
      {roiSelectorPortal}
    </>
  );
}
