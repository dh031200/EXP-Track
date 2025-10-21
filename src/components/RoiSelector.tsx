import { useState, useRef, useEffect } from 'react';
import { initScreenCapture, getScreenDimensions, type Roi } from '../lib/tauri';
import './RoiSelector.css';

interface RoiSelectorProps {
  onRoiSelected: (roi: Roi) => void;
  onCancel?: () => void;
}

interface DragState {
  isDrawing: boolean;
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
}

/**
 * ROI Selector component - overlay mode
 * Displays a transparent overlay on top of the screen for direct region selection
 */
export function RoiSelector({ onRoiSelected, onCancel }: RoiSelectorProps) {
  const [dimensions, setDimensions] = useState<[number, number]>([0, 0]);
  const [dragState, setDragState] = useState<DragState>({
    isDrawing: false,
    startX: 0,
    startY: 0,
    currentX: 0,
    currentY: 0,
  });
  const [error, setError] = useState<string | null>(null);
  const overlayRef = useRef<HTMLDivElement>(null);

  // Initialize screen capture and get dimensions
  useEffect(() => {
    const initCapture = async () => {
      try {
        await initScreenCapture();
        const dims = await getScreenDimensions();
        setDimensions(dims);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to initialize screen capture');
      }
    };

    initCapture();
  }, []);

  // Handle ESC key to cancel
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && onCancel) {
        onCancel();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onCancel]);

  const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    const overlay = overlayRef.current;
    if (!overlay) return;

    const rect = overlay.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    setDragState({
      isDrawing: true,
      startX: x,
      startY: y,
      currentX: x,
      currentY: y,
    });
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!dragState.isDrawing) return;

    const overlay = overlayRef.current;
    if (!overlay) return;

    const rect = overlay.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    setDragState(prev => ({
      ...prev,
      currentX: x,
      currentY: y,
    }));
  };

  const handleMouseUp = () => {
    if (!dragState.isDrawing) return;

    const { startX, startY, currentX, currentY } = dragState;

    // Calculate ROI bounds
    const x = Math.round(Math.min(startX, currentX));
    const y = Math.round(Math.min(startY, currentY));
    const width = Math.round(Math.abs(currentX - startX));
    const height = Math.round(Math.abs(currentY - startY));

    // Validate ROI size (minimum 10x10 pixels)
    if (width >= 10 && height >= 10) {
      const roi: Roi = { x, y, width, height };
      onRoiSelected(roi);
    } else {
      setError('ROI must be at least 10x10 pixels');
      setTimeout(() => setError(null), 3000);
    }

    setDragState(prev => ({ ...prev, isDrawing: false }));
  };

  // Calculate selection box dimensions
  const getSelectionBox = () => {
    if (!dragState.isDrawing) return null;

    const { startX, startY, currentX, currentY } = dragState;
    const x = Math.min(startX, currentX);
    const y = Math.min(startY, currentY);
    const width = Math.abs(currentX - startX);
    const height = Math.abs(currentY - startY);

    return { x, y, width, height };
  };

  const selectionBox = getSelectionBox();

  if (error) {
    return (
      <div className="roi-selector-error">
        <p>Error: {error}</p>
        {onCancel && (
          <button onClick={onCancel}>Close</button>
        )}
      </div>
    );
  }

  if (dimensions[0] === 0) {
    return (
      <div className="roi-selector-loading">
        <p>Initializing...</p>
      </div>
    );
  }

  return (
    <div
      ref={overlayRef}
      className="roi-selector-overlay"
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
      style={{ cursor: dragState.isDrawing ? 'crosshair' : 'crosshair' }}
    >
      {/* Header with instructions */}
      <div className="roi-selector-header">
        <p>Click and drag to select a region</p>
        {onCancel && (
          <button onClick={onCancel}>Cancel (ESC)</button>
        )}
      </div>

      {/* Selection rectangle */}
      {selectionBox && (
        <>
          {/* Semi-transparent dark overlay */}
          <div className="roi-overlay-mask" />

          {/* Clear area for selection */}
          <div
            className="roi-selection-box"
            style={{
              left: selectionBox.x,
              top: selectionBox.y,
              width: selectionBox.width,
              height: selectionBox.height,
            }}
          >
            {/* Dimension label */}
            <div className="roi-dimension-label">
              {Math.round(selectionBox.width)} × {Math.round(selectionBox.height)}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
