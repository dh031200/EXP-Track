import React from 'react';
import { formatNumber, formatPercentage } from '../lib/expCommands';
import { TrackingStats } from '../lib/trackingCommands';
import './ExpTrackerDisplay.css';

export interface ExpTrackerDisplayProps {
  stats: TrackingStats | null;
  isTracking: boolean;
  error: string | null;
  averageData: { label: string; value: string } | null;
  calculationMode: 'prediction' | 'per_interval';
  intervalLabel: string; // e.g., "1분", "5분", "10분", "30분", "시간"
  potionUsage: { hpPerMinute: number; mpPerMinute: number };
}

export const ExpTrackerDisplay: React.FC<ExpTrackerDisplayProps> = ({
  stats,
  isTracking,
  error,
  averageData,
  calculationMode,
  intervalLabel,
  potionUsage,
}) => {
  // Create dynamic label based on mode and interval
  const modeLabel = calculationMode === 'prediction' 
    ? `${intervalLabel} 예상` 
    : `${intervalLabel}당`;
  // Always show cards - remove empty state check for better UX
  return (
    <div className="exp-tracker-display">
      {/* 2 rows: top row has 3 cards, bottom row has 2 cards */}
      <div className="stats-grid-2x3">
        {/* Row 1 */}
        {/* Total Gained EXP - wider */}
        <div className="stat-card primary stat-card-wide">
          <div className="stat-info">
            <div className="stat-label-compact">경험치</div>
            <div className="stat-value-compact">
              {stats ? formatNumber(stats.total_exp) : '0'}
            </div>
          </div>
        </div>

        {/* Total Gained Percentage */}
        <div className="stat-card">
          <div className="stat-info">
            <div className="stat-label-compact">퍼센트</div>
            <div className="stat-value-compact">
              {stats ? formatPercentage(stats.total_percentage) : '0.00%'}
            </div>
          </div>
        </div>

        {/* HP Potion Usage */}
        <div className="stat-card stat-card-with-unit">
          <div className="stat-info">
            <div className="stat-label-compact">HP 포션</div>
            <div className="stat-value-compact">
              {stats ? stats.hp_potions_used : '0'}
            </div>
          </div>
          <div className="stat-unit-badge">
            {potionUsage.hpPerMinute.toFixed(1)}/분
          </div>
        </div>
      </div>

      {/* Row 2 - wider layout without spacer */}
      <div className="stats-grid-2x3-no-spacer">
        {/* Average based on selected interval - much wider */}
        <div className="stat-card stat-card-with-unit stat-card-wide">
          <div className="stat-info">
            <div className="stat-label-compact">{modeLabel} 경험치</div>
            <div className="stat-value-compact">
              {averageData ? averageData.value : '0'}
            </div>
          </div>
          <div className="stat-unit-badge">/{intervalLabel || '시간'}</div>
        </div>

        {/* MP Potion Usage */}
        <div className="stat-card stat-card-with-unit">
          <div className="stat-info">
            <div className="stat-label-compact">MP 포션</div>
            <div className="stat-value-compact">
              {stats ? stats.mp_potions_used : '0'}
            </div>
          </div>
          <div className="stat-unit-badge">
            {potionUsage.mpPerMinute.toFixed(1)}/분
          </div>
        </div>
      </div>
    </div>
  );
};
