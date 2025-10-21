import { useState } from 'react';
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

  return (
    <>
      {/* Backdrop */}
      <div
        className="modal-backdrop"
        onClick={() => !isSelecting && onClose()}
        style={{ display: isSelecting ? 'none' : 'block' }}
      />

      {/* Modal */}
      <div className="modal-container" style={{ display: isSelecting ? 'none' : 'flex' }}>
        <div className="modal-header">
          <h2>영역 설정</h2>
          {!isSelecting && (
            <button
              className="modal-close-btn"
              onClick={onClose}
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
