import React from 'react';
import { ExpStats, formatNumber, formatPercentage, formatElapsedTime } from '../lib/expCommands';
import './ExpTrackerDisplay.css';

export interface ExpTrackerDisplayProps {
  stats: ExpStats | null;
  level: number | null;
  exp: number | null;
  percentage: number | null;
  mapName: string | null;
  isTracking: boolean;
  error: string | null;
  ocrStatus: 'success' | 'warning' | 'error';
  averageData: { label: string; value: string } | null;
}

export const ExpTrackerDisplay: React.FC<ExpTrackerDisplayProps> = ({
  stats,
  level,
  exp,
  percentage,
  mapName,
  isTracking,
  error,
  ocrStatus,
  averageData,
}) => {
  // Don't show error message, only status indicator
  // if (error) {
  //   return (
  //     <div className="exp-tracker-error">
  //       <div className="error-icon">⚠️</div>
  //       <div className="error-message">{error}</div>
  //     </div>
  //   );
  // }

  if (!stats && !isTracking) {
    return (
      <div className="exp-tracker-empty">
        <div className="empty-icon">📊</div>
        <div className="empty-message">추적을 시작하세요</div>
      </div>
    );
  }

  return (
    <div className="exp-tracker-display">
      {/* Horizontal Statistics Grid - 3 cards */}
      <div className="stats-grid-horizontal-3">
        {/* Total Gained EXP */}
        <div className="stat-card primary">
          <div className="stat-info">
            <div className="stat-label-compact">획득 경험치</div>
            <div className="stat-value-compact">
              {stats ? formatNumber(stats.total_exp) : '0'}
            </div>
          </div>
        </div>

        {/* Total Gained Percentage */}
        <div className="stat-card">
          <div className="stat-info">
            <div className="stat-label-compact">진행률</div>
            <div className="stat-value-compact">
              {stats ? formatPercentage(stats.total_percentage) : '0.00%'}
            </div>
          </div>
        </div>

        {/* Average based on selected interval */}
        <div className="stat-card">
          <div className="stat-info">
            <div className="stat-label-compact">
              {averageData ? `평균 (${averageData.label})` : '평균'}
            </div>
            <div className="stat-value-compact">
              {averageData ? averageData.value : '0'}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
