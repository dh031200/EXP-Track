import { useState, useEffect } from 'react';
import { useSettingsStore } from '../stores/settingsStore';
import { getPotionSlotConfig, setPotionSlotConfig, VALID_SLOTS } from '../lib/configCommands';
import './Settings.css';

import hpIcon from '/icons/hp.png';
import mpIcon from '/icons/mp.png';
import slotIcon from '/icons/slot.png';

// Keyboard slot coordinates (relative to image, percentage-based)
const SLOT_POSITIONS: { [key: string]: { x: number; y: number; width: number; height: number } } = {
  shift: { x: 0, y: 0, width: 25, height: 50 },
  ins: { x: 25, y: 0, width: 25, height: 50 },
  home: { x: 50, y: 0, width: 25, height: 50 },
  pup: { x: 75, y: 0, width: 25, height: 50 },
  ctrl: { x: 0, y: 50, width: 25, height: 50 },
  del: { x: 25, y: 50, width: 25, height: 50 },
  end: { x: 50, y: 50, width: 25, height: 50 },
  pdn: { x: 75, y: 50, width: 25, height: 50 },
};

export function Settings() {
  const { backgroundOpacity, targetDuration, setBackgroundOpacity, setTargetDuration, resetSettings } = useSettingsStore();

  // Potion slot configuration state
  const [hpSlot, setHpSlot] = useState<string>('shift');
  const [mpSlot, setMpSlot] = useState<string>('ins');
  const [potionConfigError, setPotionConfigError] = useState<string | null>(null);

  // Load potion config on mount
  useEffect(() => {
    getPotionSlotConfig()
      .then(config => {
        setHpSlot(config.hp_potion_slot);
        setMpSlot(config.mp_potion_slot);
      })
      .catch(err => {
        console.error('Failed to load potion config:', err);
        setPotionConfigError('포션 설정을 불러오는데 실패했습니다');
      });
  }, []);

  const handleOpacityChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setBackgroundOpacity(parseFloat(e.target.value));
  };

  const handleDurationChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTargetDuration(parseInt(e.target.value));
  };

  const handleHpSlotChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newHpSlot = e.target.value;

    if (newHpSlot === mpSlot) {
      setPotionConfigError('HP 포션과 MP 포션은 서로 다른 칸이어야 합니다');
      return;
    }

    try {
      await setPotionSlotConfig(newHpSlot, mpSlot);
      setHpSlot(newHpSlot);
      setPotionConfigError(null);
    } catch (err) {
      console.error('Failed to save HP slot config:', err);
      setPotionConfigError('HP 포션 설정을 저장하는데 실패했습니다');
    }
  };

  const handleMpSlotChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newMpSlot = e.target.value;

    if (newMpSlot === hpSlot) {
      setPotionConfigError('HP 포션과 MP 포션은 서로 다른 칸이어야 합니다');
      return;
    }

    try {
      await setPotionSlotConfig(hpSlot, newMpSlot);
      setMpSlot(newMpSlot);
      setPotionConfigError(null);
    } catch (err) {
      console.error('Failed to save MP slot config:', err);
      setPotionConfigError('MP 포션 설정을 저장하는데 실패했습니다');
    }
  };

  const handleSlotClick = async (e: React.MouseEvent<HTMLDivElement>) => {
    e.preventDefault();
    
    const rect = e.currentTarget.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * 100;
    const y = ((e.clientY - rect.top) / rect.height) * 100;

    // Find which slot was clicked
    let clickedSlot: string | null = null;
    for (const [slot, pos] of Object.entries(SLOT_POSITIONS)) {
      if (x >= pos.x && x < pos.x + pos.width && y >= pos.y && y < pos.y + pos.height) {
        clickedSlot = slot;
        break;
      }
    }

    if (!clickedSlot) return;

    const isLeftClick = e.button === 0;
    const isRightClick = e.button === 2;

    // Helper function to get default MP slot (different from clicked HP slot)
    const getDefaultMpSlot = (hpSlot: string) => {
      const mpCandidates = ['end', 'ins', 'del', 'pdn'];
      return mpCandidates.find(slot => slot !== hpSlot) || 'end';
    };

    // Helper function to get default HP slot (different from clicked MP slot)
    const getDefaultHpSlot = (mpSlot: string) => {
      const hpCandidates = ['del', 'shift', 'ctrl', 'home'];
      return hpCandidates.find(slot => slot !== mpSlot) || 'del';
    };

    if (isLeftClick) {
      // Left click: HP potion
      if (hpSlot === clickedSlot) {
        // Cancel HP if clicking on same slot
        try {
          const defaultMp = mpSlot || getDefaultMpSlot('');
          await setPotionSlotConfig(getDefaultHpSlot(defaultMp), defaultMp);
          setHpSlot('');
          setPotionConfigError(null);
        } catch (err) {
          console.error('Failed to clear HP slot:', err);
          setPotionConfigError('HP 포션 설정 해제 실패');
        }
      } else if (mpSlot === clickedSlot) {
        // Trying to set HP where MP is already set
        setPotionConfigError('같은 위치에 설정할 수 없습니다');
        setTimeout(() => setPotionConfigError(null), 3000);
      } else {
        // Set HP to clicked slot (automatically moves from previous location)
        try {
          const finalMpSlot = mpSlot || getDefaultMpSlot(clickedSlot);
          await setPotionSlotConfig(clickedSlot, finalMpSlot);
          setHpSlot(clickedSlot);
          setPotionConfigError(null);
        } catch (err) {
          console.error('Failed to save HP slot config:', err);
          setPotionConfigError('HP 포션 설정 저장 실패');
        }
      }
    } else if (isRightClick) {
      // Right click: MP potion
      if (mpSlot === clickedSlot) {
        // Cancel MP if clicking on same slot
        try {
          const defaultHp = hpSlot || getDefaultHpSlot('');
          await setPotionSlotConfig(defaultHp, getDefaultMpSlot(defaultHp));
          setMpSlot('');
          setPotionConfigError(null);
        } catch (err) {
          console.error('Failed to clear MP slot:', err);
          setPotionConfigError('MP 포션 설정 해제 실패');
        }
      } else if (hpSlot === clickedSlot) {
        // Trying to set MP where HP is already set
        setPotionConfigError('같은 위치에 설정할 수 없습니다');
        setTimeout(() => setPotionConfigError(null), 3000);
      } else {
        // Set MP to clicked slot (automatically moves from previous location)
        try {
          const finalHpSlot = hpSlot || getDefaultHpSlot(clickedSlot);
          await setPotionSlotConfig(finalHpSlot, clickedSlot);
          setMpSlot(clickedSlot);
          setPotionConfigError(null);
        } catch (err) {
          console.error('Failed to save MP slot config:', err);
          setPotionConfigError('MP 포션 설정 저장 실패');
        }
      }
    }
  };

  const handleContextMenu = (e: React.MouseEvent<HTMLDivElement>) => {
    e.preventDefault();
  };

  const opacityPercent = Math.round(backgroundOpacity * 100);

  return (
    <div className="settings-container">
      <div className="settings-section">
        <h3>배경</h3>

        <div className="settings-item">
          <label htmlFor="opacity-slider" className="settings-label">
            투명도: <strong>{opacityPercent}%</strong>
          </label>
          <div className="slider-container">
            <span className="slider-label">30%</span>
            <input
              id="opacity-slider"
              type="range"
              min="0.3"
              max="1"
              step="0.05"
              value={backgroundOpacity}
              onChange={handleOpacityChange}
              className="opacity-slider"
            />
            <span className="slider-label">100%</span>
          </div>
          <p className="settings-help">
            앱 전체의 투명도를 조절합니다. 값이 낮을수록 더 투명해집니다.
          </p>
        </div>
      </div>

      <div className="settings-section">
        <h3>타이머</h3>

        <div className="settings-item">
          <label htmlFor="duration-select" className="settings-label">
            시간 설정
          </label>
          <select
            id="duration-select"
            value={targetDuration}
            onChange={handleDurationChange}
            className="duration-select"
          >
            <option value="0">사용 안 함</option>
            <option value="5">5분</option>
            <option value="10">10분</option>
            <option value="15">15분</option>
            <option value="30">30분</option>
            <option value="60">1시간</option>
            <option value="120">2시간</option>
            <option value="180">3시간</option>
          </select>
          <p className="settings-help">
            목표 시간을 설정하면 타이머에 완료 예정 시각이 표시됩니다.
          </p>
        </div>
      </div>

      <div className="settings-section">
        <div className="settings-header-with-error">
          <h3>포션 설정</h3>
          <div className="potion-error-inline">
            {potionConfigError && (
              <div className="settings-error-inline">
                {potionConfigError}
              </div>
            )}
          </div>
        </div>

        <div className="keyboard-slot-container">
          <div 
            className="keyboard-slot-wrapper" 
            onMouseDown={handleSlotClick}
            onContextMenu={handleContextMenu}
          >
            <img src={slotIcon} alt="Keyboard Slots" className="keyboard-slot-image" />
            
            {/* HP Potion Icon */}
            {hpSlot && SLOT_POSITIONS[hpSlot] && (
              <img 
                src={hpIcon} 
                alt="HP Potion" 
                className="potion-overlay-icon"
                style={{
                  left: `${SLOT_POSITIONS[hpSlot].x + SLOT_POSITIONS[hpSlot].width / 2}%`,
                  top: `${SLOT_POSITIONS[hpSlot].y + SLOT_POSITIONS[hpSlot].height / 2}%`,
                }}
              />
            )}
            
            {/* MP Potion Icon */}
            {mpSlot && SLOT_POSITIONS[mpSlot] && (
              <img 
                src={mpIcon} 
                alt="MP Potion" 
                className="potion-overlay-icon"
                style={{
                  left: `${SLOT_POSITIONS[mpSlot].x + SLOT_POSITIONS[mpSlot].width / 2}%`,
                  top: `${SLOT_POSITIONS[mpSlot].y + SLOT_POSITIONS[mpSlot].height / 2}%`,
                }}
              />
            )}
          </div>
          <div className="keyboard-slot-instructions">
            <p className="settings-help" style={{ marginBottom: '4px', fontWeight: '600', color: '#333' }}>
              사용 방법:
            </p>
            <p className="settings-help" style={{ marginBottom: '2px' }}>
              • <strong>좌클릭</strong>: HP 포션 설정 (같은 위치 재클릭 시 취소)
            </p>
            <p className="settings-help">
              • <strong>우클릭</strong>: MP 포션 설정 (같은 위치 재클릭 시 취소)
            </p>
          </div>
        </div>
      </div>

      {/* <div className="settings-actions">
        <button onClick={resetSettings} className="reset-button">
          기본값으로 재설정
        </button>
      </div> */}
    </div>
  );
}
