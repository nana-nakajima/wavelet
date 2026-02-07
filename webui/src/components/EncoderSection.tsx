import React, { useCallback, useRef } from 'react';
import { useTonverkStore, PAGE_LABELS, TRACK_TYPE_CONFIG } from '../tonverkStore';
import { useAudio } from '../context/AudioContext';

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
    <div className="encoder-knob-container" style={{ width: knobSize }}>
      <div
        ref={knobRef}
        className="encoder-knob"
        onMouseDown={handleMouseDown}
        style={{
          width: knobSize,
          height: knobSize,
          borderColor: color,
        }}
      >
        <div
          className="encoder-knob-indicator"
          style={{
            transform: `rotate(${rotation}deg)`,
            backgroundColor: color,
            height: size === 'small' ? '35%' : '50%',
          }}
        />
        <div className="encoder-knob-center" style={{
          width: size === 'small' ? 6 : size === 'large' ? 12 : 8,
          height: size === 'small' ? 6 : size === 'large' ? 12 : 8,
        }} />
      </div>
      <span className="encoder-knob-label" style={{ fontSize: size === 'small' ? '7px' : '8px' }}>{label}</span>
      <span className="encoder-knob-value" style={{ fontSize: size === 'small' ? '9px' : '10px' }}>{value}</span>
    </div>
  );
};

export const EncoderSection: React.FC = () => {
  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const setTrackPage = useTonverkStore(state => state.setTrackPage);
  const updateTrackParamStore = useTonverkStore(state => state.updateTrackParam);
  const { setTrackParam } = useAudio();

  const track = tracks.find(t => t.id === selectedTrackId) || tracks[0];
  const config = TRACK_TYPE_CONFIG[track.type];

  const handleParamChange = (param: string) => (value: number) => {
    updateTrackParamStore(track.id, param, value);
    setTrackParam(track.id, param, value);
  };

  const getKnobs = () => {
    switch (track.currentPage) {
      case 'trig':
        return [
          { label: 'LEN', value: track.sequencer.length, param: 'length', min: 1, max: 16 },
          { label: 'SCALE', value: track.sequencer.scale * 100, param: 'scale', min: 12, max: 200 },
          { label: 'RESET', value: 0, param: 'reset', min: 0, max: 1 },
          { label: 'CHG', value: 0, param: 'change', min: 0, max: 1 },
          { label: 'PORT', value: 0, param: 'portamento', min: 0, max: 127 },
          { label: 'MIDI CH', value: track.id, param: 'midiCh', min: 1, max: 16 },
          { label: '', value: 0, param: '', min: 0, max: 127 },
          { label: '', value: 0, param: '', min: 0, max: 127 },
        ];
      case 'src':
        return [
          { label: 'TUNE', value: track.srcParams.tune || 64, param: 'src_tune', min: 0, max: 127 },
          { label: 'MODE', value: track.srcParams.playMode || 0, param: 'src_playMode', min: 0, max: 3 },
          { label: 'LOOP', value: track.srcParams.loopCrossfade || 0, param: 'src_loopCrossfade', min: 0, max: 127 },
          { label: 'SLOT', value: track.srcParams.sampleSlot || 0, param: 'src_sampleSlot', min: 0, max: 127 },
          { label: 'STR', value: track.srcParams.strt || 0, param: 'src_strt', min: 0, max: 127 },
          { label: 'END', value: track.srcParams.end || 127, param: 'src_end', min: 0, max: 127 },
          { label: 'LSTR', value: track.srcParams.lstr || 0, param: 'src_lstr', min: 0, max: 127 },
          { label: 'LEND', value: track.srcParams.lend || 127, param: 'src_lend', min: 0, max: 127 },
        ];
      case 'fltr':
        return [
          { label: 'FREQ', value: track.fltrParams.freq || 64, param: 'fltr_freq', min: 0, max: 127 },
          { label: 'RESO', value: track.fltrParams.reso || 0, param: 'fltr_reso', min: 0, max: 127 },
          { label: 'ENV', value: track.fltrParams.envDepth || 64, param: 'fltr_envDepth', min: -64, max: 64 },
          { label: 'TYPE', value: track.fltrParams.fltrType || 0, param: 'fltr_fltrType', min: 0, max: 127 },
          { label: 'BASE', value: track.fltrParams.base || 0, param: 'fltr_base', min: 0, max: 127 },
          { label: 'WIDTH', value: track.fltrParams.width || 127, param: 'fltr_width', min: 0, max: 127 },
          { label: 'ATK', value: track.fltrParams.atk || 0, param: 'fltr_atk', min: 0, max: 127 },
          { label: 'DEC', value: track.fltrParams.dec || 64, param: 'fltr_dec', min: 0, max: 127 },
        ];
      case 'amp':
        return [
          { label: 'ATK', value: track.ampParams.atk || 0, param: 'amp_atk', min: 0, max: 127 },
          { label: 'DEC', value: track.ampParams.dec || 64, param: 'amp_dec', min: 0, max: 127 },
          { label: 'SUS', value: track.ampParams.sus || 127, param: 'amp_sus', min: 0, max: 127 },
          { label: 'REL', value: track.ampParams.rel || 64, param: 'amp_rel', min: 0, max: 127 },
          { label: 'DRV', value: track.ampParams.overdrive || 0, param: 'amp_overdrive', min: 0, max: 127 },
          { label: 'PAN', value: track.ampParams.pan || 64, param: 'amp_pan', min: 0, max: 127 },
          { label: 'VEL', value: track.ampParams.velToVol || 64, param: 'amp_velToVol', min: 0, max: 127 },
          { label: 'VOL', value: track.volume, param: 'volume', min: 0, max: 127 },
        ];
      case 'fx':
        return [
          { label: 'TYPE', value: 0, param: 'fx1_type', min: 0, max: 14 },
          { label: 'PAR1', value: 0, param: 'fx1_p1', min: 0, max: 127 },
          { label: 'PAR2', value: 0, param: 'fx1_p2', min: 0, max: 127 },
          { label: 'PAR3', value: 0, param: 'fx1_p3', min: 0, max: 127 },
          { label: 'TYPE', value: 0, param: 'fx2_type', min: 0, max: 14 },
          { label: 'PAR1', value: 0, param: 'fx2_p1', min: 0, max: 127 },
          { label: 'PAR2', value: 0, param: 'fx2_p2', min: 0, max: 127 },
          { label: 'PAR3', value: 0, param: 'fx2_p3', min: 0, max: 127 },
        ];
      case 'mod':
        return [
          { label: 'LFO1', value: track.modParams.lfo1Speed || 16, param: 'mod_lfo1Speed', min: 0, max: 127 },
          { label: 'DEP1', value: track.modParams.lfo1Dep || 64, param: 'mod_lfo1Dep', min: -64, max: 64 },
          { label: 'LFO2', value: track.modParams.lfo2Speed || 16, param: 'mod_lfo2Speed', min: 0, max: 127 },
          { label: 'DEP2', value: track.modParams.lfo2Dep || 64, param: 'mod_lfo2Dep', min: -64, max: 64 },
          { label: 'ENV ATK', value: track.modParams.modAtk || 0, param: 'mod_modAtk', min: 0, max: 127 },
          { label: 'ENV DEC', value: track.modParams.modDec || 64, param: 'mod_modDec', min: 0, max: 127 },
          { label: 'ENV SUS', value: track.modParams.modSus || 127, param: 'mod_modSus', min: 0, max: 127 },
          { label: 'ENV REL', value: track.modParams.modRel || 64, param: 'mod_modRel', min: 0, max: 127 },
        ];
      default:
        return [];
    }
  };

  const knobs = getKnobs();

  return (
    <div className="encoder-section">
      <div className="encoder-page-tabs">
        {Object.entries(PAGE_LABELS).map(([key, label]) => (
          <button
            key={key}
            className={`encoder-page-tab ${track.currentPage === key ? 'active' : ''}`}
            style={track.currentPage === key ? { color: config.color } : {}}
            onClick={() => setTrackPage(track.id, key as any)}
          >
            {label}
          </button>
        ))}
      </div>

      <div className="encoder-knobs-row">
        {knobs.map((knob, i) => (
          <Knob
            key={i}
            value={knob.value}
            min={knob.min}
            max={knob.max}
            label={knob.label}
            onChange={handleParamChange(knob.param)}
            color={config.color}
            size="medium"
          />
        ))}
      </div>
    </div>
  );
};
