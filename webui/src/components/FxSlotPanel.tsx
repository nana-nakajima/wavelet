import React from 'react';
import { useTonverkStore, TRACK_TYPE_CONFIG } from '../tonverkStore';

const FX_TYPES = [
  'BYPASS', 'CHRONO PITCH', 'COMB FILTER', 'COMPRESSOR',
  'DEGRADER', 'DIRTSHAPER', 'FILTERBANK', 'INFINITE FLANGER',
  'LOWPASS', 'PANORAMIC CHORUS', 'PHASE 98', 'SATURATOR DELAY',
  'FREQUENCY WARP', 'SUPERVOID', 'WARBLE'
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

  if (!fxSlot) return null;

  return (
    <div className={`fx-slot ${fxSlot.bypass ? 'bypass' : 'active'}`} style={{ borderColor: color }}>
      <div className="fx-slot-header">
        <span className="fx-slot-label">FX{slot}</span>
        <span className={`fx-bypass-indicator ${fxSlot.bypass ? 'on' : 'off'}`}>
          {fxSlot.bypass ? 'BYP' : 'ON'}
        </span>
      </div>
      <div className="fx-slot-type">{fxSlot.type.toUpperCase()}</div>
      <div className="fx-slot-params">
        {Object.entries(fxSlot.params).slice(0, 4).map(([key, value]) => (
          <div key={key} className="fx-param-mini">
            <span className="fx-param-label">{key.slice(0, 3)}</span>
            <span className="fx-param-value">{Math.round(value)}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export const FxSlotPanel: React.FC = () => {
  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const setTracks = useTonverkStore(state => state.setTracks);

  const track = tracks.find(t => t.id === selectedTrackId) || tracks[0];
  const config = TRACK_TYPE_CONFIG[track.type];
  const maxSlots = track.type === 'audio' || track.type === 'bus' ? 2 : 1;

  const updateFxSlot = (slotId: number, updates: Partial<{ type: string; bypass: boolean }>) => {
    setTracks([{
      id: track.id,
      fxSlots: track.fxSlots.map(slot =>
        slot.id === slotId ? { ...slot, ...updates } : slot
      ) as any,
    }]);
  };

  const toggleBypass = (slotId: number) => {
    const slot = track.fxSlots.find(s => s.id === slotId);
    if (slot) {
      updateFxSlot(slotId, { bypass: !slot.bypass });
    }
  };

  return (
    <div className="fx-slot-panel">
      <div className="fx-panel-header">INSERT FX</div>
      <div className="fx-slots-row">
        {Array.from({ length: maxSlots }, (_, i) => {
          const slotId = i + 1;
          const fxSlot = track.fxSlots.find(s => s.id === slotId);
          return (
            <div key={slotId} className="fx-slot-wrapper">
              <div
                className={`fx-slot ${fxSlot?.bypass ? 'bypass' : 'active'}`}
                style={{ borderColor: config.color }}
                onClick={() => toggleBypass(slotId)}
              >
                <div className="fx-slot-header">
                  <span className="fx-slot-label">FX{slotId}</span>
                  <span className={`fx-bypass-indicator ${fxSlot?.bypass ? 'on' : 'off'}`}>
                    {fxSlot?.bypass ? 'BYP' : 'ON'}
                  </span>
                </div>
                <div className="fx-slot-type">
                  {fxSlot?.type ? fxSlot.type.toUpperCase() : 'BYPASS'}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
