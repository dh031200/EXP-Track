import { useState, useRef, useEffect } from 'react';
import { RoiSelector } from './RoiSelector';
import { captureRegion, bytesToDataUrl, maximizeWindowForROI, restoreWindow, initScreenCapture, setAlwaysOnTop, type Roi } from '../lib/tauri';
import { useRoiStore } from '../stores/roiStore';

interface WindowState {
  width: number;
  height: number;
  x: number;
  y: number;
}

interface RoiDemoProps {
  onSelectingChange?: (isSelecting: boolean) => void;
}

/**
 * Demo component showing ROI selector and captured region
 */
export function RoiDemo({ onSelectingChange }: RoiDemoProps) {
  const [isSelecting, setIsSelecting] = useState(false);
  const [selectedRoi, setSelectedRoi] = useState<Roi | null>(null);
  const [capturedImage, setCapturedImage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const windowStateRef = useRef<WindowState | null>(null);

  // ROI store for persistence
  const { setRoi: saveRoiToStore } = useRoiStore();

  // Initialize screen capture on component mount
  useEffect(() => {
    const init = async () => {
      console.log('🟢 [RoiDemo] Starting screen capture initialization...');
      await initScreenCapture();
      console.log('🟢 [RoiDemo] ✅ Screen capture initialized successfully');
      setIsInitialized(true);
    };

    // Execute initialization - NO ERROR HANDLING for debugging
    init();
  }, []);

  const handleSelectRegionClick = async () => {
    // DEBUG: NO ERROR HANDLING - Let errors crash for debugging
    console.log('🔵 [RoiDemo] User clicked "Select Region" button');
    console.log('🔵 [RoiDemo] Starting window maximize...');

    // Set window to always on top for overlay
    await setAlwaysOnTop(true);

    windowStateRef.current = await maximizeWindowForROI();

    console.log('🔵 [RoiDemo] Window maximized successfully:', windowStateRef.current);
    setIsSelecting(true);
    onSelectingChange?.(true);
  };

  const handleRoiSelected = async (roi: Roi) => {
    console.log('🟢 [RoiDemo] ROI selected:', roi);
    setSelectedRoi(roi);

    // Save ROI to persistent storage (for demo, using 'level' type)
    try {
      await saveRoiToStore('level', roi);
      console.log('🟢 [RoiDemo] ROI saved to persistent storage');
    } catch (err) {
      console.error('⚠️ [RoiDemo] Failed to save ROI:', err);
      // Don't fail the whole operation if persistence fails
    }

    // Restore window BEFORE hiding overlay
    console.log('🟢 [RoiDemo] Restoring window settings...');
    await setAlwaysOnTop(false);

    if (windowStateRef.current) {
      console.log('🟢 [RoiDemo] Restoring window to original state:', windowStateRef.current);
      await restoreWindow(windowStateRef.current);
      windowStateRef.current = null;
    }

    // Small delay to let window restore before hiding overlay
    await new Promise(resolve => setTimeout(resolve, 100));

    setIsSelecting(false);
    onSelectingChange?.(false);

    try {
      const bytes = await captureRegion(roi);
      const dataUrl = bytesToDataUrl(bytes);
      setCapturedImage(dataUrl);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to capture region');
      setCapturedImage(null);
    }
  };

  const handleCancel = async () => {
    console.log('🔴 [RoiDemo] Cancel clicked');

    // Restore window settings
    await setAlwaysOnTop(false);

    // Restore window size and position on cancel
    if (windowStateRef.current) {
      console.log('🔴 [RoiDemo] Restoring window to original state:', windowStateRef.current);
      await restoreWindow(windowStateRef.current);
      windowStateRef.current = null;
    }

    // Small delay to let window restore before hiding overlay
    await new Promise(resolve => setTimeout(resolve, 100));

    setIsSelecting(false);
    onSelectingChange?.(false);
  };

  return (
    <div style={{
      padding: isSelecting ? '0' : '20px',
      maxWidth: isSelecting ? 'none' : '800px',
      margin: '0 auto',
      // Completely transparent when selecting
      backgroundColor: 'transparent',
      minHeight: isSelecting ? '100vh' : 'auto',
      // Hide all content when selecting
      display: isSelecting ? 'contents' : 'block'
    }}>
      {!isSelecting && <h2>ROI Selector Demo</h2>}

      <div style={{ marginBottom: '20px', visibility: isSelecting ? 'hidden' : 'visible' }}>
        <button
          onClick={handleSelectRegionClick}
          disabled={isSelecting || !isInitialized}
          style={{
            padding: '10px 20px',
            fontSize: '16px',
            background: (!isInitialized || isSelecting) ? '#ccc' : '#2196F3',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: (isSelecting || !isInitialized) ? 'not-allowed' : 'pointer',
          }}
        >
          {!isInitialized ? 'Initializing...' : 'Select Region'}
        </button>
        <p style={{ fontSize: '12px', color: '#666', marginTop: '8px' }}>
          {!isInitialized
            ? 'Initializing screen capture...'
            : 'Window will maximize to screen size for accurate selection'}
        </p>
      </div>

      {error && !isSelecting && (
        <div
          style={{
            padding: '10px',
            background: '#ffebee',
            border: '1px solid #f44336',
            borderRadius: '4px',
            marginBottom: '20px',
          }}
        >
          <strong>Error:</strong> {error}
        </div>
      )}

      {selectedRoi && !isSelecting && (
        <div style={{ marginBottom: '20px' }}>
          <h3>Selected ROI:</h3>
          <pre style={{ background: '#f5f5f5', padding: '10px', borderRadius: '4px' }}>
            {JSON.stringify(selectedRoi, null, 2)}
          </pre>
        </div>
      )}

      {capturedImage && !isSelecting && (
        <div>
          <h3>Captured Image:</h3>
          <div
            style={{
              border: '1px solid #ddd',
              borderRadius: '4px',
              padding: '10px',
              background: '#f9f9f9',
            }}
          >
            <img
              src={capturedImage}
              alt="Captured region"
              style={{ maxWidth: '100%', display: 'block' }}
            />
          </div>
        </div>
      )}

      {isSelecting && (
        <RoiSelector onRoiSelected={handleRoiSelected} onCancel={handleCancel} />
      )}
    </div>
  );
}
