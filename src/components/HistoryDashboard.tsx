import { useSessionStore } from '../stores/sessionStore';
import { useTimerSettingsStore } from '../stores/timerSettingsStore';
import './HistoryDashboard.css';

interface HistoryDashboardProps {
  isOpen: boolean;
  onClose: () => void;
}

export function HistoryDashboard({ isOpen, onClose }: HistoryDashboardProps) {
  const {
    sessions,
    getTotalSessions,
    getTotalTrackingTime,
    getAverageDuration,
    getRecentSessions,
    deleteSession,
    clearAllSessions,
  } = useSessionStore();

  const {
    showTotalTime,
    showSessionCount,
  } = useTimerSettingsStore();

  if (!isOpen) return null;

  const recentSessions = getRecentSessions(10);
  const totalSessions = getTotalSessions();
  const totalTime = getTotalTrackingTime();
  const avgDuration = getAverageDuration();

  const formatTime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const formatDate = (timestamp: number): string => {
    const date = new Date(timestamp);
    return date.toLocaleDateString('ko-KR', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const calculateAverage = (intervalMinutes: number): string => {
    if (sessions.length === 0) return '0';

    const intervalSeconds = intervalMinutes * 60;
    const totalExp = sessions.reduce((sum, s) => sum + (s.expGained || 0), 0);
    const totalDuration = sessions.reduce((sum, s) => sum + s.duration, 0);

    if (totalDuration === 0) return '0';

    const expPerSecond = totalExp / totalDuration;
    const avgExp = Math.floor(expPerSecond * intervalSeconds);

    return avgExp.toLocaleString();
  };

  const handleClearAll = () => {
    if (confirm('모든 세션 기록을 삭제하시겠습니까?')) {
      clearAllSessions();
    }
  };

  // When used in a separate window, don't show backdrop or close button
  const isInSeparateWindow = typeof window !== 'undefined' && window.location.pathname === '/history';

  return (
    <>
      {!isInSeparateWindow && <div className="modal-backdrop" onClick={onClose} />}

      <div className={isInSeparateWindow ? 'history-dashboard fullscreen' : 'modal-container history-dashboard'}>
        {!isInSeparateWindow && (
          <div className="modal-header">
            <h2>사냥 기록</h2>
            <button
              className="modal-close-btn"
              onClick={onClose}
              title="닫기"
            >
              ×
            </button>
          </div>
        )}

        <div className={isInSeparateWindow ? '' : 'modal-content'}>
          {/* Statistics Overview */}
          <div className="stats-overview">
            {showSessionCount && (
              <div className="history-stat-card">
                <div className="stat-label">총 세션</div>
                <div className="stat-value">{totalSessions}</div>
              </div>
            )}

            {showTotalTime && (
              <div className="history-stat-card">
                <div className="stat-label">총 사냥 시간</div>
                <div className="stat-value">{formatTime(totalTime)}</div>
              </div>
            )}

            {totalSessions > 0 && (
              <div className="history-stat-card">
                <div className="stat-label">평균 세션 시간</div>
                <div className="stat-value">{formatTime(avgDuration)}</div>
              </div>
            )}
          </div>

          {/* Average Exp Calculations - All intervals shown */}
          <div className="averages-section">
            <h3 className="section-title">평균 경험치 (예정)</h3>
            <div className="averages-grid">
              <div className="average-card">
                <div className="average-label">5분</div>
                <div className="average-value">{calculateAverage(5)}</div>
              </div>
              <div className="average-card">
                <div className="average-label">10분</div>
                <div className="average-value">{calculateAverage(10)}</div>
              </div>
              <div className="average-card">
                <div className="average-label">30분</div>
                <div className="average-value">{calculateAverage(30)}</div>
              </div>
              <div className="average-card">
                <div className="average-label">1시간</div>
                <div className="average-value">{calculateAverage(60)}</div>
              </div>
            </div>
          </div>

          {/* Recent Sessions List */}
          <div className="sessions-section">
            <div className="section-header">
              <h3 className="section-title">최근 세션</h3>
              {sessions.length > 0 && (
                <button
                  className="clear-all-btn"
                  onClick={handleClearAll}
                >
                  전체 삭제
                </button>
              )}
            </div>

            {recentSessions.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">📊</div>
                <div className="empty-text">아직 기록된 세션이 없습니다</div>
                <div className="empty-subtext">추적을 시작하고 리셋하면 세션이 저장됩니다</div>
              </div>
            ) : (
              <div className="sessions-list">
                {recentSessions.map((session) => (
                  <div key={session.id} className="session-item">
                    <div className="session-info">
                      <div className="session-time">
                        {formatDate(session.startTime)}
                      </div>
                      <div className="session-duration">
                        {formatTime(session.duration)}
                      </div>
                    </div>
                    <button
                      className="session-delete-btn"
                      onClick={() => deleteSession(session.id)}
                      title="삭제"
                    >
                      🗑️
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  );
}
