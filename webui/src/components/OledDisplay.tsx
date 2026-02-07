import React from 'react';
import { useTonverkStore, PAGE_LABELS } from '../tonverkStore';

export const OledDisplay: React.FC = () => {
  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const transport = useTonverkStore(state => state.transport);

  const track = tracks.find(t => t.id === selectedTrackId) || tracks[0];

  const formatPageName = () => {
    return PAGE_LABELS[track.currentPage];
  };

  return (
    <div className="oled-display-panel">
      <div className="oled-screen">
        <div className="oled-header">
          <span className="oled-track-name">{track.name}</span>
          <span className="oled-page-name">{formatPageName()}</span>
        </div>

        <div className="oled-main">
          <div className="oled-param-row">
            {track.currentPage === 'src' && (
              <>
                <div className="oled-param">
                  <span className="oled-label">TUNE</span>
                  <span className="oled-value">{(track.srcParams.tune || 64) - 64}</span>
                </div>
                <div className="oled-param">
                  <span className="oled-label">MODE</span>
                  <span className="oled-value">{['FWD', 'REV', 'FLOD', 'RLOD'][track.srcParams.playMode || 0] || 'FWD'}</span>
                </div>
              </>
            )}
            {track.currentPage === 'fltr' && (
              <>
                <div className="oled-param">
                  <span className="oled-label">FREQ</span>
                  <span className="oled-value">{track.fltrParams.freq || 64}</span>
                </div>
                <div className="oled-param">
                  <span className="oled-label">RES</span>
                  <span className="oled-value">{track.fltrParams.reso || 0}</span>
                </div>
              </>
            )}
            {track.currentPage === 'amp' && (
              <>
                <div className="oled-param">
                  <span className="oled-label">ATK</span>
                  <span className="oled-value">{track.ampParams.atk || 0}</span>
                </div>
                <div className="oled-param">
                  <span className="oled-label">DEC</span>
                  <span className="oled-value">{track.ampParams.dec || 64}</span>
                </div>
              </>
            )}
            {track.currentPage === 'fx' && (
              <>
                <div className="oled-param">
                  <span className="oled-label">FX1</span>
                  <span className="oled-value">{track.fxSlots[0]?.type?.toUpperCase() || 'BYP'}</span>
                </div>
                <div className="oled-param">
                  <span className="oled-label">FX2</span>
                  <span className="oled-value">{track.fxSlots[1]?.type?.toUpperCase() || 'BYP'}</span>
                </div>
              </>
            )}
            {track.currentPage === 'mod' && (
              <>
                <div className="oled-param">
                  <span className="oled-label">LFO1</span>
                  <span className="oled-value">{track.modParams.lfo1Speed || 16}</span>
                </div>
                <div className="oled-param">
                  <span className="oled-label">LFO2</span>
                  <span className="oled-value">{track.modParams.lfo2Speed || 16}</span>
                </div>
              </>
            )}
            {track.currentPage === 'trig' && (
              <>
                <div className="oled-param">
                  <span className="oled-label">LEN</span>
                  <span className="oled-value">{track.sequencer.length}</span>
                </div>
                <div className="oled-param">
                  <span className="oled-label">STP</span>
                  <span className="oled-value">{transport.currentStep % 16 + 1}</span>
                </div>
              </>
            )}
          </div>

          <div className="oled-waveform">
            <svg viewBox="0 0 128 32" className="waveform-svg">
              <polyline
                points="0,16 16,16 32,16 48,16 64,16 80,16 96,16 112,16 128,16"
                fill="none"
                stroke="#00ff88"
                strokeWidth="1"
              />
            </svg>
          </div>

          <div className="oled-footer">
            <span className="oled-tempo">{Math.round(transport.tempo)} BPM</span>
            <span className="oled-position">{String.fromCharCode(65 + transport.patternBank)}:{String(transport.patternId + 1).padStart(3, '0')}</span>
          </div>
        </div>
      </div>
    </div>
  );
};
