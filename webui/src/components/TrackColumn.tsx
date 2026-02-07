import React, { useCallback, useRef, useEffect } from 'react';
import { useTonverkStore, TRACK_TYPE_CONFIG } from '../tonverkStore';
import type { TrackType } from '../tonverkStore';
import { useAudio } from '../context/AudioContext';

interface TrackColumnProps {
  trackId: number;
}

export const TrackColumn: React.FC<TrackColumnProps> = ({ trackId }) => {
  const track = useTonverkStore(state => state.tracks[trackId - 1]);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const setSelectedTrack = useTonverkStore(state => state.setSelectedTrack);
  const toggleMuteStore = useTonverkStore(state => state.toggleMute);
  const toggleSoloStore = useTonverkStore(state => state.toggleSolo);
  const { setTrackMute, setTrackSolo } = useAudio();

  const isSelected = selectedTrackId === trackId;
  const config = TRACK_TYPE_CONFIG[track.type as TrackType];

  const handleMute = () => {
    toggleMuteStore(trackId);
    setTrackMute(trackId, !track.muted);
  };

  const handleSolo = () => {
    toggleSoloStore(trackId);
    setTrackSolo(trackId, !track.solo);
  };

  const getCurrentPageLabel = () => {
    const labels: Record<string, string> = {
      trig: 'TRIG',
      src: 'SRC',
      fltr: 'FLTR',
      amp: 'AMP',
      fx: 'FX',
      mod: 'MOD',
    };
    return labels[track.currentPage] || 'TRIG';
  };

  const getKeyParam = () => {
    switch (track.currentPage) {
      case 'src':
        return `T: ${(track.srcParams.tune || 64) - 64}`;
      case 'fltr':
        return `F: ${track.fltrParams.freq || 64}`;
      case 'amp':
        return `A: ${track.ampParams.atk || 0}`;
      default:
        return '';
    }
  };

  return (
    <div
      className={`track-strip ${isSelected ? 'selected' : ''}`}
      style={{ borderColor: config.color }}
      onClick={() => setSelectedTrack(trackId)}
    >
      <div className="track-strip-header" style={{ backgroundColor: config.color }}>
        <span className="track-strip-name">{track.id}</span>
      </div>

      <div className="track-strip-controls">
        <button
          className={`track-strip-btn mute ${track.muted ? 'active' : ''}`}
          onClick={(e) => { e.stopPropagation(); handleMute(); }}
          style={{ color: track.muted ? '#ff0000' : '#666' }}
        >
          M
        </button>
        <button
          className={`track-strip-btn solo ${track.solo ? 'active' : ''}`}
          onClick={(e) => { e.stopPropagation(); handleSolo(); }}
          style={{ color: track.solo ? '#ffff00' : '#666' }}
        >
          S
        </button>
      </div>

      <div className="track-strip-page" style={{ color: config.color }}>
        {getCurrentPageLabel()}
      </div>

      <div className="track-strip-param">
        {getKeyParam()}
      </div>

      <div className="track-strip-volume">
        <div className="volume-bar">
          <div
            className="volume-fill"
            style={{
              height: `${(track.volume / 127) * 100}%`,
              backgroundColor: config.color,
            }}
          />
        </div>
        <span className="volume-label">{Math.round((track.volume / 127) * 100)}%</span>
      </div>
    </div>
  );
};

export const TransportBar: React.FC = () => {
  const transport = useTonverkStore(state => state.transport);
  const setTransport = useTonverkStore(state => state.setTransport);
  const nextStep = useTonverkStore(state => state.nextStep);
  const { play, stop, setTempo } = useAudio();

  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (transport.playing) {
      const beatDuration = 60000 / transport.tempo;
      interval = setInterval(nextStep, beatDuration / 4);
    }
    return () => clearInterval(interval);
  }, [transport.playing, transport.tempo, nextStep]);

  const handlePlayStop = () => {
    setTransport({ playing: !transport.playing });
    if (transport.playing) {
      stop();
    } else {
      play();
    }
  };

  const handleRecord = () => {
    setTransport({ recording: !transport.recording });
  };

  const handleRestart = () => {
    setTransport({ playing: false, currentStep: 0 });
    stop();
  };

  const handleTempoChange = (value: number) => {
    setTransport({ tempo: value });
    setTempo(value);
  };

  return (
    <div className="transport-bar">
      <div className="transport-left">
        <button
          className={`transport-btn play-btn ${transport.playing ? 'active' : ''}`}
          onClick={handlePlayStop}
        >
          {transport.playing ? 'STOP' : 'PLAY'}
        </button>

        <button
          className={`transport-btn record-btn ${transport.recording ? 'active' : ''}`}
          onClick={handleRecord}
        >
          REC
        </button>

        <button
          className="transport-btn"
          onClick={handleRestart}
        >
          RESTART
        </button>
      </div>

      <div className="tempo-display">
        <Knob
          value={Math.round(transport.tempo)}
          min={20}
          max={300}
          label="BPM"
          color="#00ff88"
          size="small"
          onChange={handleTempoChange}
        />
        <span className="tempo-label">BPM</span>
      </div>

      <div className="step-counter">
        <div className="step-indicators">
          {Array.from({ length: 16 }, (_, i) => (
            <div
              key={i}
              className={`step-indicator ${i === transport.currentStep % 16 ? 'active' : ''}`}
              style={i === transport.currentStep % 16 ? { backgroundColor: transport.playing ? '#00ff88' : '#333' } : {}}
            />
          ))}
        </div>
        <span className="step-label">{transport.currentPage + 1}:{transport.currentStep % 16 + 1}</span>
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

  const trackGroups = [
    { type: 'audio', name: 'AUDIO', range: [1, 8], color: '#00ff88' },
    { type: 'bus', name: 'BUS', range: [9, 12], color: '#00d4ff' },
    { type: 'send', name: 'SEND', range: [13, 15], color: '#bf5fff' },
    { type: 'mix', name: 'MIX', range: [16, 16], color: '#ffcc00' },
  ];

  return (
    <div className="track-selector">
      {trackGroups.map(group => (
        <div key={group.name} className="track-group">
          <div className="track-group-label" style={{ color: group.color }}>{group.name}</div>
          {Array.from({ length: group.range[1] - group.range[0] + 1 }, (_, i) => {
            const trackNum = group.range[0] + i;
            return (
              <button
                key={trackNum}
                className={`track-selector-btn ${selectedTrackId === trackNum ? 'selected' : ''}`}
                style={{
                  backgroundColor: group.color,
                  opacity: selectedTrackId === trackNum ? 1 : 0.4,
                  boxShadow: selectedTrackId === trackNum ? `0 0 12px ${group.color}` : 'none',
                }}
                onClick={() => setSelectedTrack(trackNum)}
              >
                {trackNum}
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
};

// Mini Knob component for TransportBar
interface KnobProps {
  value: number;
  min?: number;
  max?: number;
  label: string;
  onChange: (value: number) => void;
  color?: string;
  size?: 'small' | 'medium' | 'large';
}

const Knob: React.FC<KnobProps> = ({
  value,
  min = 0,
  max = 127,
  label,
  onChange,
  color = '#00ff88',
  size = 'medium'
}) => {
  const knobRef = useRef<HTMLDivElement>(null);
  const startY = useRef<number>(0);
  const startValue = useRef<number>(0);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    startY.current = e.clientY;
    startValue.current = value;

    const handleMouseMove = (e: MouseEvent) => {
      const delta = startY.current - e.clientY;
      const range = max - min;
      const sensitivity = size === 'small' ? 400 : size === 'large' ? 100 : 200;
      const newValue = Math.max(min, Math.min(max, startValue.current + (delta / sensitivity) * range));
      onChange(Math.round(newValue));
    };

    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [value, min, max, onChange, size]);

  const percentage = (value - min) / (max - min);
  const rotation = -135 + percentage * 270;
  const knobSize = size === 'small' ? 28 : size === 'large' ? 56 : 40;

  return (
    <div className="knob-container" style={{ width: knobSize }}>
      <div
        ref={knobRef}
        className="knob"
        onMouseDown={handleMouseDown}
        style={{
          width: knobSize,
          height: knobSize,
          borderColor: color,
        }}
      >
        <div
          className="knob-indicator"
          style={{
            transform: `rotate(${rotation}deg)`,
            backgroundColor: color,
            height: size === 'small' ? '35%' : '50%',
          }}
        />
        <div className="knob-center" style={{
          width: size === 'small' ? 6 : size === 'large' ? 12 : 8,
          height: size === 'small' ? 6 : size === 'large' ? 12 : 8,
        }} />
      </div>
      <span className="knob-label" style={{ fontSize: size === 'small' ? '7px' : '8px' }}>{label}</span>
      <span className="knob-value" style={{ fontSize: size === 'small' ? '9px' : '10px' }}>{value}</span>
    </div>
  );
};
