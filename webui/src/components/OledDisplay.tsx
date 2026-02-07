import React, { useEffect, useRef } from 'react';
import { useTonverkStore } from '../tonverkStore';
import { useAudio } from '../context/AudioContext';
import { audioEngine } from '../audio/engine';

export const OledDisplay: React.FC = () => {
  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const transport = useTonverkStore(state => state.transport);
  const { isReady } = useAudio();

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>(0);

  const track = tracks.find(t => t.id === selectedTrackId) || tracks[0];

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const drawWaveform = () => {
      const width = canvas.width;
      const height = canvas.height;

      // Clear canvas
      ctx.fillStyle = '#0a0a0c';
      ctx.fillRect(0, 0, width, height);

      // Draw grid
      ctx.strokeStyle = '#1a1a1a';
      ctx.lineWidth = 1;
      for (let x = 0; x < width; x += 16) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, height);
        ctx.stroke();
      }
      for (let y = 0; y < height; y += 16) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(width, y);
        ctx.stroke();
      }

      if (isReady) {
        const waveform = audioEngine.getWaveformData();
        if (waveform && waveform.length > 0) {
          ctx.strokeStyle = '#00ff88';
          ctx.lineWidth = 1.5;
          ctx.beginPath();

          const sliceWidth = width / Math.min(waveform.length, 1024);
          let x = 0;

          for (let i = 0; i < Math.min(waveform.length, 1024); i++) {
            const v = (waveform[i] + 1) / 2;
            const y = v * height;

            if (i === 0) {
              ctx.moveTo(x, y);
            } else {
              ctx.lineTo(x, y);
            }
            x += sliceWidth;
          }

          ctx.stroke();
        }
      } else {
        // Draw placeholder waveform
        ctx.strokeStyle = '#333';
        ctx.lineWidth = 1;
        ctx.setLineDash([4, 4]);
        ctx.beginPath();
        ctx.moveTo(0, height / 2);
        ctx.lineTo(width, height / 2);
        ctx.stroke();
        ctx.setLineDash([]);
      }

      animationRef.current = requestAnimationFrame(drawWaveform);
    };

    drawWaveform();

    return () => {
      cancelAnimationFrame(animationRef.current);
    };
  }, [isReady]);

  const formatPageName = () => {
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
                  <span className="oled-value">{(['FWD', 'REV', 'FLOD', 'RLOD'])[track.srcParams.playMode || 0] || 'FWD'}</span>
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

          <canvas
            ref={canvasRef}
            width={360}
            height={40}
            className="oled-waveform-canvas"
          />

          <div className="oled-footer">
            <span className="oled-tempo">{Math.round(transport.tempo)} BPM</span>
            <span className="oled-position">{String.fromCharCode(65 + transport.patternBank)}:{String(transport.patternId + 1).padStart(3, '0')}</span>
          </div>
        </div>
      </div>
    </div>
  );
};
