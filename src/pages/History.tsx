import { HistoryDashboard } from '../components/HistoryDashboard';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function History() {
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

  return (
    <div style={{
      width: '100vw',
      height: '100vh',
      background: 'transparent',
      overflow: 'hidden',
      display: 'flex',
      flexDirection: 'column',
    }}>
      {/* Titlebar matching main window */}
      <div
        onMouseDown={handleDragStart}
        style={{
          height: '44px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: 'rgba(255, 255, 255, 0.98)',
          borderBottom: '1px solid rgba(0, 0, 0, 0.08)',
          borderTopLeftRadius: '12px',
          borderTopRightRadius: '12px',
          cursor: 'grab',
          userSelect: 'none',
          flexShrink: 0,
        }}
      >
        {/* Title text */}
        <div style={{
          fontSize: '12px',
          fontWeight: '500',
          color: 'rgba(0, 0, 0, 0.5)',
          pointerEvents: 'none'
        }}>
          사냥 기록
        </div>

        {/* Window controls */}
        <div
          onMouseDown={(e) => e.stopPropagation()}
          style={{
            position: 'absolute',
            top: '6px',
            right: '12px',
            display: 'flex',
            gap: '8px',
          }}
        >
          <button
            onClick={handleMinimize}
            style={{
              width: '32px',
              height: '32px',
              borderRadius: '8px',
              border: 'none',
              background: 'rgba(0, 0, 0, 0.4)',
              color: '#fff',
              fontSize: '20px',
              fontWeight: '300',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.15s ease',
              paddingBottom: '4px',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'rgba(0, 0, 0, 0.6)';
              e.currentTarget.style.transform = 'scale(1.05)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'rgba(0, 0, 0, 0.4)';
              e.currentTarget.style.transform = 'scale(1)';
            }}
            title="Minimize"
          >
            −
          </button>
          <button
            onClick={handleClose}
            style={{
              width: '32px',
              height: '32px',
              borderRadius: '8px',
              border: 'none',
              background: 'rgba(255, 59, 48, 0.8)',
              color: '#fff',
              fontSize: '20px',
              fontWeight: '300',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.15s ease',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = '#ff3b30';
              e.currentTarget.style.transform = 'scale(1.05)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'rgba(255, 59, 48, 0.8)';
              e.currentTarget.style.transform = 'scale(1)';
            }}
            title="Close"
          >
            ×
          </button>
        </div>
      </div>

      {/* Content */}
      <div style={{
        flex: 1,
        overflow: 'hidden',
        background: 'rgba(255, 255, 255, 0.98)',
        borderBottomLeftRadius: '12px',
        borderBottomRightRadius: '12px',
      }}>
        <HistoryDashboard isOpen={true} onClose={() => {}} />
      </div>
    </div>
  );
}
