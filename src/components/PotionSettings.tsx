import { useState, useEffect } from 'react';
import { getPotionSlotConfig, setPotionSlotConfig } from '../lib/configCommands';
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

export function PotionSettings() {
  const [hpSlot, setHpSlot] = useState<string>('shift');
  const [mpSlot, setMpSlot] = useState<string>('ins');
  const [potionConfigError, setPotionConfigError] = useState<string | null>(null);

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

  const handleSlotClick = async (e: React.MouseEvent<HTMLDivElement>) => {
    e.preventDefault();
    
    const rect = e.currentTarget.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * 100;
    const y = ((e.clientY - rect.top) / rect.height) * 100;

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

    const getDefaultMpSlot = (hpSlot: string) => {
      const mpCandidates = ['end', 'ins', 'del', 'pdn'];
      return mpCandidates.find(slot => slot !== hpSlot) || 'end';
    };

    try {
      if (isLeftClick) {
        const newMpSlot = clickedSlot === hpSlot ? getDefaultMpSlot(hpSlot) : mpSlot;
        await setPotionSlotConfig(clickedSlot, newMpSlot);
        setHpSlot(clickedSlot);
        setMpSlot(newMpSlot);
        setPotionConfigError(null);
      } else if (isRightClick) {
        const newHpSlot = clickedSlot === mpSlot ? getDefaultMpSlot(clickedSlot) : hpSlot;
        await setPotionSlotConfig(newHpSlot, clickedSlot);
        setHpSlot(newHpSlot);
        setMpSlot(clickedSlot);
        setPotionConfigError(null);
      }
    } catch (err) {
      console.error('Failed to save potion config:', err);
      setPotionConfigError('포션 설정을 저장하는데 실패했습니다');
    }
  };

  const handleContextMenu = (e: React.MouseEvent<HTMLDivElement>) => {
    e.preventDefault();
  };

  return (
    <div className="settings-container">
      <div className="settings-section">
        <div className="settings-header-with-error">
          <h3>포션 슬롯 설정</h3>
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
            <p><strong>좌클릭</strong>: HP 포션 슬롯 지정</p>
            <p><strong>우클릭</strong>: MP 포션 슬롯 지정</p>
          </div>
        </div>
      </div>
    </div>
  );
}

