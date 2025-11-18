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

// Import example images
import levelExample from '/icons/level_roi_example.png';
import expExample from '/icons/exp_roi_example.png';
import potionExample from '/icons/potion_roi_example.png';

const ROI_CONFIGS = [
  { type: 'level' as RoiType, label: '레벨', icon: lvIcon, color: '#4CAF50', autoDetect: true, example: levelExample },
  { type: 'exp' as RoiType, label: '경험치', icon: expIcon, color: '#2196F3', autoDetect: false, example: expExample },
  { type: 'inventory' as RoiType, label: '포션', icon: [hpIcon, mpIcon], color: '#FF5722', autoDetect: true, example: potionExample },
  // { type: 'mapLocation' as RoiType, label: 'Map', icon: '🗺️', color: '#9C27B0' }, // Commented out temporarily
  // { type: 'meso' as RoiType, label: 'Meso', icon: '💰', color: '#FF9800' }, // Commented out temporarily
];

export function CompactRoiManager({ onSelectingChange }: CompactRoiManagerProps) {
  const [isSelecting, setIsSelecting] = useState(false);
  const [currentRoiType, setCurrentRoiType] = useState<RoiType | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [previewImage, setPreviewImage] = useState<string | null>(null);
  const [previewRoiType, setPreviewRoiType] = useState<RoiType | null>(null);
  const [showExampleImage, setShowExampleImage] = useState(false);
  const [isAutoDetecting, setIsAutoDetecting] = useState(false);
  const [autoDetectError, setAutoDetectError] = useState<string | null>(null);
  const windowStateRef = useRef<WindowState | null>(null);

  const { levelRoi, expRoi, inventoryRoi, setRoi, removeRoi, loadAllRois, getLevelBoxes, setLevelWithBoxes } = useRoiStore();

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

  const handleManualSelect = (type: RoiType) => {
    const config = ROI_CONFIGS.find(c => c.type === type);
    if (!config) return;

    // Show example image for manual selection
    setCurrentRoiType(type);
    setPreviewImage(config.example);
    setPreviewRoiType(type);
    setShowExampleImage(true);
  };

  const handleSelectClick = async (type: RoiType) => {
    setCurrentRoiType(type);
    setShowExampleImage(false);
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

    // Step 5: Capture the clean screen and show preview
    try {
      const bytes = await captureRegion(roi);
      const dataUrl = bytesToDataUrl(bytes);

      // Save to temp folder via Tauri command
      await invoke('save_roi_preview', {
        roiType: roiType,
        imageData: dataUrl.split(',')[1], // Remove data:image/png;base64, prefix
      });

      // Update preview with captured image
      setPreviewImage(dataUrl);
      setPreviewRoiType(roiType);
      setShowExampleImage(false);
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

      // For level ROI, use matched box coordinates if available (like inventory)
      let captureRoi = roi;
      if (type === 'level') {
        const levelBoxes = getLevelBoxes();
        if (levelBoxes && levelBoxes.length > 0) {
          // Calculate bounding box from all matched digit boxes
          const minX = Math.min(...levelBoxes.map(b => b.x));
          const minY = Math.min(...levelBoxes.map(b => b.y));
          const maxX = Math.max(...levelBoxes.map(b => b.x + b.width));
          const maxY = Math.max(...levelBoxes.map(b => b.y + b.height));

          // Add padding (10 pixels on each side)
          const padding = 10;
          captureRoi = {
            x: Math.max(0, minX - padding),
            y: Math.max(0, minY - padding),
            width: maxX - minX + padding * 2,
            height: maxY - minY + padding * 2,
          };
        }
      }

      // Capture the region in real-time
      const bytes = await captureRegion(captureRoi);
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
    setShowExampleImage(false);
    setCurrentRoiType(null);
  };

  const handleRemoveRoi = async (type: RoiType) => {
    await removeRoi(type);
  };

  const handleAutoDetect = async (type: RoiType) => {
    setIsAutoDetecting(true);
    setAutoDetectError(null);
    try {
      console.log(`🔍 Auto-detecting ${type} ROI...`);
      const result = await autoDetectRois();
      console.log(`📊 Auto-detect result for ${type}:`, result);

      if (type === 'level' && result.level) {
        if (result.level_boxes && result.level_boxes.length > 0) {
          await setLevelWithBoxes(result.level, result.level_boxes);
          console.log(`✅ Level ROI auto-detected with ${result.level_boxes.length} digit boxes`);
        } else {
          await setRoi('level', result.level);
          console.log('✅ Level ROI auto-detected');
        }
        await handleViewPreview('level');
      } else if (type === 'inventory' && result.inventory) {
        await setRoi('inventory', result.inventory);
        console.log('✅ Inventory ROI auto-detected');
        await handleViewPreview('inventory');
      } else {
        const errorMsg = `${type} ROI를 자동으로 찾지 못했습니다. 수동으로 선택해주세요.`;
        console.warn(`⚠️ ${errorMsg}`);
        setAutoDetectError(errorMsg);
        setTimeout(() => {
          handleManualSelect(type);
        }, 2000);
      }
    } catch (err) {
      const errorMsg = `자동 감지 실패: ${err instanceof Error ? err.message : String(err)}`;
      console.error(`❌ Failed to auto-detect ${type}:`, err);
      setAutoDetectError(errorMsg);
      setTimeout(() => {
        setAutoDetectError(null);
        handleManualSelect(type);
      }, 2000);
    } finally {
      setIsAutoDetecting(false);
    }
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
                const isRoiValid = isConfigured && roi && roi.width > 0 && roi.height > 0;

                // Determine button behavior
                const handleButtonClick = () => {
                  if (isRoiValid) {
                    // Valid ROI exists (auto or manual): show preview
                    handleViewPreview(type);
                  } else if (autoDetect) {
                    // Auto-detect enabled but ROI not found: retry auto-detect
                    handleAutoDetect(type);
                  } else {
                    // Manual only: show example and manual select
                    handleManualSelect(type);
                  }
                };

                const getButtonTitle = () => {
                  if (isRoiValid) {
                    return `${label} 미리보기`;
                  }
                  if (autoDetect) {
                    return `${label} 자동 감지 시도 (클릭)`;
                  }
                  return `${label} 영역 선택 (예시 보기)`;
                };

                return (
                  <div key={type} className="roi-button-group">
                    <button
                      onClick={handleButtonClick}
                      disabled={!isInitialized || isAutoDetecting}
                      className="roi-select-btn"
                      style={{ borderColor: color }}
                      title={getButtonTitle()}
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
                      {autoDetect ? (
                        isRoiValid ? (
                          <span className="roi-auto-badge">자동</span>
                        ) : (
                          <span className="roi-auto-badge-warning">미감지</span>
                        )
                      ) : (
                        isConfigured && <span className="roi-check">✓</span>
                      )}
                    </button>
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
            
            {isAutoDetecting && (
              <div className="roi-init-status">
                <span className="spinner-small"></span>
                <span>자동 감지 중...</span>
              </div>
            )}

            {autoDetectError && (
              <div className="roi-init-status roi-init-error">
                <span>⚠️ {autoDetectError}</span>
              </div>
            )}
          </div>

          {/* Preview Container */}
          {previewImage && (
            <div className={`roi-preview-container ${previewImage ? 'slide-in' : ''}`}>
              <div className="roi-preview-header">
                <span className="roi-preview-title">
                  {showExampleImage ? '예시' : '미리보기'} - {getLabelForType(previewRoiType!)}
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
              {showExampleImage && (
                <div className="roi-example-actions">
                  <div className="roi-example-description">
                    <div className="roi-example-title">📌 캡처 방법</div>
                    <div className="roi-example-text">
                      {previewRoiType === 'exp' && '경험치 바 전체(숫자 + 바 + 퍼센트)를 포함하도록 드래그하여 선택하세요.'}
                      {previewRoiType === 'level' && '레벨 숫자 부분을 드래그하여 선택하세요.'}
                      {previewRoiType === 'inventory' && '퀵슬롯 전체 영역을 드래그하여 선택하세요.'}
                    </div>
                  </div>
                  <button
                    onClick={() => handleSelectClick(currentRoiType!)}
                    className="roi-select-manual-btn"
                  >
                    영역 선택 시작
                  </button>
                </div>
              )}
              {!showExampleImage && (
                <div className="roi-preview-actions">
                  <button
                    onClick={() => handleManualSelect(previewRoiType!)}
                    className="roi-manual-reselect-btn"
                  >
                    수동으로 다시 선택
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Render RoiSelector as Portal to document.body */}
      {roiSelectorPortal}
    </>
  );
}
