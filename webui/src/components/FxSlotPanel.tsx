import React from 'react';
import { useTonverkStore, TRACK_TYPE_CONFIG } from '../tonverkStore';
import { useAudio } from '../context/AudioContext';

const FX_TYPES = [
  { id: 0, name: 'BYPASS', params: [] },
  { id: 1, name: 'CHRONO PITCH', params: ['TUNE', 'WIN', 'FDBK', 'DEP', 'HPF', 'LPF', 'SPD', 'MIX'] },
  { id: 2, name: 'COMB FILTER', params: ['SPD', 'DEP', 'SPH', 'DTUN', 'FREQ', 'FDBK', 'LPF', 'MIX'] },
  { id: 3, name: 'COMPRESSOR', params: ['THR', 'ATK', 'REL', 'MUP', 'RAT', 'SCS', 'SCF', 'MIX'] },
  { id: 4, name: 'DEGRADER', params: ['BR', 'OVER', 'SRR', 'DROP', 'RATE', 'DEP', 'FREZ', 'F.TIM'] },
  { id: 5, name: 'DIRTSHAPER', params: ['DRV', 'RECT', 'HPF', 'LPF', 'NOIS', 'N.FRQ', 'N.RES', 'MIX'] },
  { id: 6, name: 'FILTERBANK', params: ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'] },
  { id: 7, name: 'INFINITE FLANGER', params: ['SPD', 'DEP', 'TUNE', 'FDBK', 'LPF'] },
  { id: 8, name: 'LOWPASS', params: ['SPD', 'DEP', 'SPH', 'LAG', 'FREQ', 'RESO', 'SPRD'] },
  { id: 9, name: 'PANORAMIC CHORUS', params: ['DEP', 'SPD', 'HPF', 'WDTH', 'MIX'] },
  { id: 10, name: 'PHASE 98', params: ['SPD', 'DEP', 'SHP', 'LAG', 'FREQ', 'FDBK', 'STG', 'MIX'] },
  { id: 11, name: 'SATURATOR DELAY', params: ['TIME', 'X', 'WID', 'FDBK', 'HPF', 'LPF', 'MIX'] },
  { id: 12, name: 'REVERB', params: ['PRE', 'DEC', 'DAMP', 'SIZE', 'LOWC', 'HIGHC'] },
  { id: 13, name: 'CHORUS', params: ['RATE', 'DEPTH', 'MIX'] },
  { id: 14, name: 'DELAY', params: ['TIME', 'FDBK', 'MIX'] },
];

interface FxSlotProps {
  slot: number;
  trackId: number;
  color: string;
}

const FxSlot: React.FC<FxSlotProps> = ({ slot, trackId, color }) => {
  const tracks = useTonverkStore(state => state.tracks);
  const track = tracks.find(t => t.id === trackId) || tracks[0];
  const fxSlot = track.fxSlots.find(s => s.id === slot);
  const setTracks = useTonverkStore(state => state.setTracks);

  const fxType = FX_TYPES.find(f => f.name === (fxSlot?.type?.toUpperCase() || 'BYPASS')) || FX_TYPES[0];

  const updateFxSlot = (updates: Partial<{ type: string; bypass: boolean; params: Record<string, number> }>) => {
    setTracks([{
      id: track.id,
      fxSlots: track.fxSlots.map(s =>
        s.id === slot ? { ...s, ...updates } : s
      ) as any,
    }]);
  };

  const cycleFxType = () => {
    const currentIndex = FX_TYPES.findIndex(f => f.name === fxSlot?.type?.toUpperCase());
    const nextIndex = (currentIndex + 1) % FX_TYPES.length;
    updateFxSlot({ type: FX_TYPES[nextIndex].name.toLowerCase() });
  };

  const toggleBypass = () => {
    if (fxSlot) {
      updateFxSlot({ bypass: !fxSlot.bypass });
    }
  };

  if (!fxSlot) return null;

  return (
    <div className={`fx-slot ${fxSlot.bypass ? 'bypass' : 'active'}`} style={{ borderColor: color }}>
      <div className="fx-slot-header">
        <span className="fx-slot-label">FX{slot}</span>
        <button
          className={`fx-bypass-indicator ${fxSlot.bypass ? 'on' : 'off'}`}
          onClick={toggleBypass}
        >
          {fxSlot.bypass ? 'BYP' : 'ON'}
        </button>
      </div>

      <div className="fx-slot-type" onClick={cycleFxType} style={{ cursor: 'pointer' }}>
        {fxSlot.type.toUpperCase()}
      </div>

      {!fxSlot.bypass && fxType.params.length > 0 && (
        <div className="fx-slot-params">
          {fxType.params.slice(0, 4).map((param, i) => {
            const paramValue = fxSlot.params?.[param.toLowerCase()] || 64;
            return (
              <div key={param} className="fx-param-mini">
                <span className="fx-param-label">{param}</span>
                <div className="fx-param-bar">
                  <div
                    className="fx-param-fill"
                    style={{
                      width: `${(paramValue / 127) * 100}%`,
                      backgroundColor: color,
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

export const FxSlotPanel: React.FC = () => {
  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);

  const track = tracks.find(t => t.id === selectedTrackId) || tracks[0];
  const config = TRACK_TYPE_CONFIG[track.type as 'audio' | 'bus' | 'send' | 'mix'];
  const maxSlots = track.type === 'audio' || track.type === 'bus' ? 2 : 1;

  return (
    <div className="fx-slot-panel">
      <div className="fx-panel-header">INSERT FX</div>
      <div className="fx-slots-row">
        {Array.from({ length: maxSlots }, (_, i) => {
          const slotId = i + 1;
          return (
            <div key={slotId} className="fx-slot-wrapper">
              <FxSlot slot={slotId} trackId={track.id} color={config.color} />
            </div>
          );
        })}
      </div>
    </div>
  );
};
