import React, { useEffect, useRef, useCallback } from 'react';
import { useTonverkStore, PAGE_LABELS, TRACK_TYPE_CONFIG } from '../tonverkStore';

interface KnobProps {
  value: number;
  min?: number;
  max?: number;
  label: string;
  onChange: (value: number) => void;
  color?: string;
}

export const Knob: React.FC<KnobProps> = ({ value, min = 0, max = 127, label, onChange, color = '#00ff88' }) => {
  const knobRef = useRef<HTMLDivElement>(null);
  const startY = useRef<number>(0);
  const startValue = useRef<number>(0);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    startY.current = e.clientY;
    startValue.current = value;

    const handleMouseMove = (e: MouseEvent) => {
      const delta = startY.current - e.clientY;
      const range = max - min;
      const sensitivity = 200;
      const newValue = Math.max(min, Math.min(max, startValue.current + (delta / sensitivity) * range));
      onChange(newValue);
    };

    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [value, min, max, onChange]);

  const percentage = (value - min) / (max - min);
  const rotation = -135 + percentage * 270;

  return (
    <div className="knob-container">
      <div
        ref={knobRef}
        className="knob"
        onMouseDown={handleMouseDown}
        style={{
          borderColor: color,
        }}
      >
        <div
          className="knob-indicator"
          style={{
            transform: `rotate(${rotation}deg)`,
            backgroundColor: color,
          }}
        />
      </div>
      <span className="knob-label">{label}</span>
      <span className="knob-value">{Math.round(value)}</span>
    </div>
  );
};

interface StepButtonProps {
  active: boolean;
  trigType: 'none' | 'note' | 'lock' | 'combined';
  color: string;
  onClick: () => void;
}

export const StepButton: React.FC<StepButtonProps> = ({ active, trigType, color, onClick }) => {
  const getLedColor = () => {
    if (!active) return '#1a1a1a';
    switch (trigType) {
      case 'note': return '#ff0000';
      case 'lock': return '#ffff00';
      case 'combined': return '#ff4444';
      default: return '#1a1a1a';
    }
  };

  return (
    <button
      className={`step-button ${active ? 'active' : ''}`}
      onClick={onClick}
      style={{
        backgroundColor: getLedColor(),
        boxShadow: active ? `0 0 10px ${color}` : 'none',
      }}
    />
  );
};

interface TrackColumnProps {
  trackId: number;
}

export const TrackColumn: React.FC<TrackColumnProps> = ({ trackId }) => {
  const track = useTonverkStore(state => state.tracks[trackId - 1]);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const setSelectedTrack = useTonverkStore(state => state.setSelectedTrack);
  const setTrackPage = useTonverkStore(state => state.setTrackPage);
  const toggleMute = useTonverkStore(state => state.toggleMute);
  const toggleSolo = useTonverkStore(state => state.toggleSolo);
  const transport = useTonverkStore(state => state.transport);

  const isSelected = selectedTrackId === trackId;
  const config = TRACK_TYPE_CONFIG[track.type];

  return (
    <div
      className={`track-column ${isSelected ? 'selected' : ''}`}
      style={{ borderColor: config.color }}
      onClick={() => setSelectedTrack(trackId)}
    >
      <div className="track-header" style={{ backgroundColor: config.color }}>
        <span className="track-name">{track.name}</span>
        <div className="track-controls">
          <button
            className={`mute-btn ${track.muted ? 'active' : ''}`}
            onClick={(e) => { e.stopPropagation(); toggleMute(trackId); }}
          >
            M
          </button>
          <button
            className={`solo-btn ${track.solo ? 'active' : ''}`}
            onClick={(e) => { e.stopPropagation(); toggleSolo(trackId); }}
          >
            S
          </button>
        </div>
      </div>

      <div className="page-tabs">
        {Object.entries(PAGE_LABELS).map(([key, label]) => (
          <button
            key={key}
            className={`page-tab ${track.currentPage === key ? 'active' : ''}`}
            onClick={(e) => { e.stopPropagation(); setTrackPage(trackId, key as any); }}
          >
            {label}
          </button>
        ))}
      </div>

      <div className="step-grid">
        {track.sequencer.pages[transport.currentPage].steps.map((step, i) => (
          <StepButton
            key={i}
            active={step.trigType !== 'none'}
            trigType={step.trigType}
            color={config.color}
            onClick={() => {}}
          />
        ))}
      </div>

      <div className="track-volume">
        <Knob
          value={track.volume}
          max={127}
          label="VOL"
          color={config.color}
          onChange={() => {}}
        />
      </div>
    </div>
  );
};

export const TransportBar: React.FC = () => {
  const transport = useTonverkStore(state => state.transport);
  const setTransport = useTonverkStore(state => state.setTransport);
  const nextStep = useTonverkStore(state => state.nextStep);

  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (transport.playing) {
      const beatDuration = 60000 / transport.tempo;
      interval = setInterval(nextStep, beatDuration / 4);
    }
    return () => clearInterval(interval);
  }, [transport.playing, transport.tempo, nextStep]);

  return (
    <div className="transport-bar">
      <button
        className={`transport-btn ${transport.playing ? 'active' : ''}`}
        onClick={() => setTransport({ playing: !transport.playing })}
      >
        {transport.playing ? 'STOP' : 'PLAY'}
      </button>

      <button
        className={`transport-btn ${transport.recording ? 'active recording' : ''}`}
        onClick={() => setTransport({ recording: !transport.recording })}
      >
        REC
      </button>

      <div className="tempo-display">
        <span className="tempo-value">{Math.round(transport.tempo)}</span>
        <span className="tempo-label">BPM</span>
      </div>

      <div className="step-display">
        <span className="step-value">
          {transport.currentPage + 1}:{transport.currentStep % 16 + 1}
        </span>
        <span className="step-label">STEP</span>
      </div>

      <div className="pattern-display">
        <span className="pattern-bank">{String.fromCharCode(65 + transport.patternBank)}</span>
        <span className="pattern-id">{String(transport.patternId + 1).padStart(3, '0')}</span>
      </div>
    </div>
  );
};

export const TrackSelector: React.FC = () => {
  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const setSelectedTrack = useTonverkStore(state => state.setSelectedTrack);

  return (
    <div className="track-selector">
      {tracks.map((track) => (
        <button
          key={track.id}
          className={`track-selector-btn ${selectedTrackId === track.id ? 'selected' : ''}`}
          style={{
            backgroundColor: TRACK_TYPE_CONFIG[track.type].color,
            opacity: selectedTrackId === track.id ? 1 : 0.5,
          }}
          onClick={() => setSelectedTrack(track.id)}
        >
          {track.id}
        </button>
      ))}
    </div>
  );
};
