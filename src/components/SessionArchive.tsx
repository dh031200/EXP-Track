import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { formatKoreanNumber } from '../lib/expCommands';
import './SessionArchive.css';

export interface SessionRecord {
  id: string;
  title: string;
  timestamp: number;
  combat_time: number; // seconds
  exp_gained: number;
  current_level: number;
  avg_exp_per_second: number;
  hp_potions_used: number;
  mp_potions_used: number;
}

interface SessionArchiveProps {
  currentSession: {
    elapsed_seconds: number;
    total_exp: number;
    level: number;
    exp_per_second: number;
    hp_potions_used: number;
    mp_potions_used: number;
  } | null;
}

export function SessionArchive({ currentSession }: SessionArchiveProps) {
  const [sessions, setSessions] = useState<SessionRecord[]>([]);
  const [sessionName, setSessionName] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState('');
  const [showCurrentSession, setShowCurrentSession] = useState(!!currentSession);

  useEffect(() => {
    loadSessions();
  }, []);

  const loadSessions = async () => {
    try {
      const loaded = await invoke<SessionRecord[]>('get_session_records');
      setSessions(loaded);
    } catch (err) {
      console.error('Failed to load sessions:', err);
    }
  };

  const handleSaveSession = async () => {
    if (!currentSession) return;

    const now = Date.now();
    const date = new Date(now);
    const defaultTitle = `${date.getFullYear()}년 ${String(date.getMonth() + 1).padStart(2, '0')}월 ${String(date.getDate()).padStart(2, '0')}일 ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')} 전투`;

    const record: SessionRecord = {
      id: now.toString(),
      title: sessionName.trim() || defaultTitle,
      timestamp: now,
      combat_time: currentSession.elapsed_seconds,
      exp_gained: currentSession.total_exp,
      current_level: currentSession.level,
      avg_exp_per_second: currentSession.exp_per_second,
      hp_potions_used: currentSession.hp_potions_used,
      mp_potions_used: currentSession.mp_potions_used,
    };

    try {
      await invoke('save_session_record', { record });
      await loadSessions();
      setSessionName('');
      setShowCurrentSession(false);
    } catch (err) {
      console.error('Failed to save session:', err);
    }
  };

  const handleDeleteSession = async (id: string) => {
    try {
      await invoke('delete_session_record', { id });
      await loadSessions();
    } catch (err) {
      console.error('Failed to delete session:', err);
    }
  };

  const handleStartEditTitle = (id: string, currentTitle: string) => {
    setEditingId(id);
    setEditingTitle(currentTitle);
  };

  const handleCancelEditTitle = () => {
    setEditingId(null);
    setEditingTitle('');
  };

  const handleSaveTitle = async (id: string) => {
    if (!editingTitle.trim()) {
      handleCancelEditTitle();
      return;
    }

    try {
      await invoke('update_session_title', { id, newTitle: editingTitle.trim() });
      await loadSessions();
      setEditingId(null);
      setEditingTitle('');
    } catch (err) {
      console.error('Failed to update session title:', err);
    }
  };

  const formatTime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours}시간 ${minutes}분 ${secs}초`;
  };

  const formatDate = (timestamp: number): string => {
    const date = new Date(timestamp);
    return `${date.getMonth() + 1}/${date.getDate()} ${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`;
  };

  return (
    <div className="session-archive">

      {/* Current Session Save Section */}
      {showCurrentSession && currentSession && (
        <div className="current-session-card">
          <h3>현재 세션</h3>
          <div className="session-stats-grid">
            <div className="stat-item">
              <span className="stat-label">전투 시간</span>
              <span className="stat-value">{formatTime(currentSession.elapsed_seconds)}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">획득 경험치</span>
              <span className="stat-value">{formatKoreanNumber(currentSession.total_exp)}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">현재 레벨</span>
              <span className="stat-value">Lv.{currentSession.level}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">평균 (1초당)</span>
              <span className="stat-value">{formatKoreanNumber(Math.floor(currentSession.exp_per_second))}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">HP 포션</span>
              <span className="stat-value">{currentSession.hp_potions_used}개</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">MP 포션</span>
              <span className="stat-value">{currentSession.mp_potions_used}개</span>
            </div>
          </div>
          <div className="save-session-form">
            <input 
              type="text"
              className="session-name-input"
              placeholder="제목 입력 (선택, 기본: 날짜/시간)"
              value={sessionName}
              onChange={(e) => setSessionName(e.target.value)}
            />
            <button 
              className="save-session-btn" 
              onClick={handleSaveSession}
              disabled={currentSession.elapsed_seconds === 0}
            >
              현재 전투 기록 저장
            </button>
          </div>
        </div>
      )}

      {/* Saved Sessions List */}
      <div className="saved-sessions">
        <h3>저장된 기록 ({sessions.length})</h3>
        {sessions.length === 0 ? (
          <div className="empty-sessions">
            <p>저장된 기록이 없습니다</p>
          </div>
        ) : (
          <div className="sessions-list">
            {sessions.map((session) => (
              <div key={session.id} className="session-record-card">
                <div className="session-record-header">
                  <div className="session-title-section">
                    {editingId === session.id ? (
                      <div className="title-edit-wrapper">
                        <input 
                          type="text"
                          className="title-edit-input"
                          value={editingTitle}
                          onChange={(e) => setEditingTitle(e.target.value)}
                          onKeyDown={(e) => {
                            if (e.key === 'Enter') handleSaveTitle(session.id);
                            if (e.key === 'Escape') handleCancelEditTitle();
                          }}
                          autoFocus
                        />
                        <button 
                          className="title-save-btn"
                          onClick={() => handleSaveTitle(session.id)}
                          title="저장"
                        >
                          ✓
                        </button>
                        <button 
                          className="title-cancel-btn"
                          onClick={handleCancelEditTitle}
                          title="취소"
                        >
                          ✕
                        </button>
                      </div>
                    ) : (
                      <div className="title-display-wrapper">
                        <h4 className="session-title">{session.title || '제목 없음'}</h4>
                        <button 
                          className="edit-title-btn"
                          onClick={() => handleStartEditTitle(session.id, session.title || '제목 없음')}
                          title="제목 수정"
                        >
                          ✏️
                        </button>
                      </div>
                    )}
                    <span className="session-timestamp">{formatDate(session.timestamp)}</span>
                  </div>
                  <button 
                    className="delete-session-btn"
                    onClick={() => handleDeleteSession(session.id)}
                    title="삭제"
                  >
                    🗑️
                  </button>
                </div>
                <div className="session-stats-grid">
                  <div className="stat-item">
                    <span className="stat-label">전투 시간</span>
                    <span className="stat-value">{formatTime(session.combat_time)}</span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">획득 경험치</span>
                    <span className="stat-value">{formatKoreanNumber(session.exp_gained)}</span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">레벨</span>
                    <span className="stat-value">Lv.{session.current_level}</span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">평균 (1초당)</span>
                    <span className="stat-value">{formatKoreanNumber(Math.floor(session.avg_exp_per_second))}</span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">HP 포션</span>
                    <span className="stat-value">{session.hp_potions_used}개</span>
                  </div>
                  <div className="stat-item">
                    <span className="stat-label">MP 포션</span>
                    <span className="stat-value">{session.mp_potions_used}개</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

