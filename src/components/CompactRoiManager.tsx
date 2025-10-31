import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { RoiSelector } from './RoiSelector';
import {
  captureRegion,
  bytesToDataUrl,
  maximizeWindowForROI,
  restoreWindow,
  initScreenCapture,
  autoDetectRois,
  setAlwaysOnTop,
  type Roi,
} from '../lib/tauri';
import { useRoiStore, type RoiType } from '../stores/roiStore';
import { invoke } from '@tauri-apps/api/core';
import './CompactRoiManager.css';

// Import icons
import lvIcon from '/icons/lv.png';
import expIcon from '/icons/exp.png';
import hpIcon from '/icons/hp.png';
import mpIcon from '/icons/mp.png';

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
  { type: 'level' as RoiType, label: '레벨', icon: lvIcon, color: '#4CAF50', autoDetect: true },
  { type: 'exp' as RoiType, label: '경험치', icon: expIcon, color: '#2196F3', autoDetect: false },
  { type: 'inventory' as RoiType, label: '포션', icon: [hpIcon, mpIcon], color: '#FF5722', autoDetect: true },
  // { type: 'mapLocation' as RoiType, label: 'Map', icon: '🗺️', color: '#9C27B0' }, // Commented out temporarily
  // { type: 'meso' as RoiType, label: 'Meso', icon: '💰', color: '#FF9800' }, // Commented out temporarily
];

export function CompactRoiManager({ onSelectingChange }: CompactRoiManagerProps) {
  const [isSelecting, setIsSelecting] = useState(false);
  const [currentRoiType, setCurrentRoiType] = useState<RoiType | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [previewImage, setPreviewImage] = useState<string | null>(null);
  const [previewRoiType, setPreviewRoiType] = useState<RoiType | null>(null);
  const windowStateRef = useRef<WindowState | null>(null);

  const { levelRoi, expRoi, inventoryRoi, setRoi, removeRoi, loadAllRois } = useRoiStore();

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
      case 'inventory': return inventoryRoi;
      // case 'mapLocation': return mapLocationRoi; // Commented out temporarily
      default: return null;
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
    if (windowStateRef.current) {
      await restoreWindow(windowStateRef.current);
      windowStateRef.current = null;
    }

    // Step 3: Restore always on top
    await setAlwaysOnTop(true);

    // Step 4: Wait 500ms for UI to settle
    await new Promise((resolve) => setTimeout(resolve, 500));

    // Step 5: Capture the clean screen
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
    if (windowStateRef.current) {
      await restoreWindow(windowStateRef.current);
      windowStateRef.current = null;
    }

    // Restore always on top
    await setAlwaysOnTop(true);

    await new Promise((resolve) => setTimeout(resolve, 100));
    setIsSelecting(false);
    setCurrentRoiType(null);
    onSelectingChange?.(false);
  };

  const handleViewPreview = async (type: RoiType) => {
    try {
      // Get the ROI configuration for this type
      const roi = getRoi(type);

      if (!roi) {
        console.error('No ROI configured for type:', type);
        return;
      }

      // Capture the region in real-time
      const bytes = await captureRegion(roi);
      const dataUrl = bytesToDataUrl(bytes);

      setPreviewImage(dataUrl);
      setPreviewRoiType(type);
    } catch (err) {
      console.error('Failed to capture preview:', err);
    }
  };

  const handleClosePreview = () => {
    setPreviewImage(null);
    setPreviewRoiType(null);
  };

  const handleRemoveRoi = async (type: RoiType) => {
    await removeRoi(type);
  };

  // Render RoiSelector outside modal container using Portal
  const roiSelectorPortal = isSelecting && currentRoiType ? createPortal(
    <RoiSelector onRoiSelected={handleRoiSelected} onCancel={handleCancel} />,
    document.body
  ) : null;

  const getLabelForType = (type: RoiType): string => {
    const config = ROI_CONFIGS.find(c => c.type === type);
    return config?.label || '';
  };

  return (
    <>
      <div className="compact-roi-manager">
        <div className="compact-roi-manager-wrapper">
          {/* Buttons Container */}
          <div className={`roi-buttons-container ${previewImage ? 'slide-out' : ''}`}>
            <div className="roi-buttons">
              {ROI_CONFIGS.map(({ type, label, icon, color, autoDetect }) => {
                const roi = getRoi(type);
                const isConfigured = roi !== null;

                return (
                  <div key={type} className="roi-button-group">
                    <button
                      onClick={() => autoDetect ? handleViewPreview(type) : handleSelectClick(type)}
                      disabled={!isInitialized}
                      className="roi-select-btn"
                      style={{ borderColor: color }}
                      title={autoDetect ? `${label} 자동 탐지 결과 보기` : `${label} 영역 ${isConfigured ? '재' : ''}선택`}
                    >
                      {Array.isArray(icon) ? (
                        <div className="roi-icon-stack">
                          <img src={icon[0]} alt="HP" className="roi-icon-img-stacked roi-icon-img-back" />
                          <img src={icon[1]} alt="MP" className="roi-icon-img-stacked roi-icon-img-front" />
                        </div>
                      ) : typeof icon === 'string' && icon.length <= 2 ? (
                        <span style={{ fontSize: '24px' }}>{icon}</span>
                      ) : (
                        <img src={icon as string} alt={label} className="roi-icon-img" />
                      )}
                      <span className="roi-label">{label}</span>
                      {autoDetect ? <span className="roi-auto-badge">자동</span> : isConfigured && <span className="roi-check">✓</span>}
                    </button>

                    {isConfigured && !autoDetect && (
                      <div className="roi-actions-compact">
                        <button
                          onClick={() => handleViewPreview(type)}
                          className="roi-action-btn view"
                          title="미리보기"
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

          {/* Preview Container */}
          {previewImage && (
            <div className={`roi-preview-container ${previewImage ? 'slide-in' : ''}`}>
              <div className="roi-preview-header">
                <span className="roi-preview-title">
                  👁️ {getLabelForType(previewRoiType!)}
                </span>
                <button
                  onClick={handleClosePreview}
                  className="roi-preview-back"
                >
                  ← 돌아가기
                </button>
              </div>
              <div className="roi-preview-image-wrapper">
                <img 
                  src={previewImage} 
                  alt={`${getLabelForType(previewRoiType!)} preview`}
                  className="roi-preview-image"
                />
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Render RoiSelector as Portal to document.body */}
      {roiSelectorPortal}
    </>
  );
}
