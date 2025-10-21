import { getCurrentWindow } from '@tauri-apps/api/window';
import './TitleBar.css';

/**
 * Custom title bar with window controls
 */
export function TitleBar() {
  const handleClose = async () => {
    const window = getCurrentWindow();
    await window.close();
  };

  const handleMinimize = async () => {
    const window = getCurrentWindow();
    await window.minimize();
  };

  const handleDragStart = async () => {
    const window = getCurrentWindow();
    await window.startDragging();
  };

  return (
    <div className="titlebar" onMouseDown={handleDragStart}>
      <div className="titlebar-title">EXP Tracker</div>
      <div className="titlebar-controls">
        <button
          className="titlebar-button minimize"
          onClick={(e) => {
            e.stopPropagation();
            handleMinimize();
          }}
          title="Minimize"
        >
          −
        </button>
        <button
          className="titlebar-button close"
          onClick={(e) => {
            e.stopPropagation();
            handleClose();
          }}
          title="Close"
        >
          ×
        </button>
      </div>
    </div>
  );
}
