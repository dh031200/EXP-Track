import { useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { CompactRoiManager } from './CompactRoiManager';
import './RoiConfigModal.css';

interface RoiConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectingChange?: (isSelecting: boolean) => void;
}

export function RoiConfigModal({ isOpen, onClose, onSelectingChange }: RoiConfigModalProps) {
  const [isSelecting, setIsSelecting] = useState(false);

  if (!isOpen) return null;

  const handleSelectingChange = (selecting: boolean) => {
    setIsSelecting(selecting);
    onSelectingChange?.(selecting);
  };

  const handleDragStart = async (e: React.MouseEvent) => {
    e.preventDefault();
    const window = getCurrentWindow();
    await window.startDragging();
  };

  return (
    <>
      {/* Modal */}
      <div className="modal-container" style={{ display: isSelecting ? 'none' : 'flex' }}>
        <div className="modal-header" onMouseDown={handleDragStart}>
          <h2>영역 설정</h2>
          {!isSelecting && (
            <button
              className="modal-close-btn"
              onClick={onClose}
              onMouseDown={(e) => e.stopPropagation()}
              title="닫기"
            >
              ×
            </button>
          )}
        </div>

        <div className="modal-content">
          <CompactRoiManager onSelectingChange={handleSelectingChange} />
        </div>
      </div>
    </>
  );
}
