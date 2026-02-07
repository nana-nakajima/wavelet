import React, { useEffect } from 'react';
import { TrackColumn, TransportBar, TrackSelector } from './components/TrackColumn';
import { EncoderSection } from './components/EncoderSection';
import { OledDisplay } from './components/OledDisplay';
import { StepGrid } from './components/StepGrid';
import { FxSlotPanel } from './components/FxSlotPanel';
import { PianoKeyboard } from './components/PianoKeyboard';
import { useTonverkStore } from './tonverkStore';
import { useAudio } from './context/AudioContext';

const App: React.FC = () => {
  const tracks = useTonverkStore(state => state.tracks);
  const viewMode = useTonverkStore(state => state.viewMode);
  const performMode = useTonverkStore(state => state.performMode);
  const togglePerformMode = useTonverkStore(state => state.togglePerformMode);
  const setViewMode = useTonverkStore(state => state.setViewMode);

  const { isConnected, latency, peakLevel, init } = useAudio();

  useEffect(() => {
    init();
  }, [init]);

  return (
    <div className={`wavelet-app ${performMode ? 'perform-mode' : ''}`}>
      <header className="app-header">
        <h1>WAVELET</h1>
        <span className="version">TONVERK</span>
      </header>

      <div className="main-content">
        <aside className="left-panel">
          <TrackSelector />
        </aside>

        <div className="center-panel">
          <TransportBar />
          <OledDisplay />
          <FxSlotPanel />
          <StepGrid />
          <PianoKeyboard />
        </div>

        <aside className="right-panel">
          <div className="view-mode-selector">
            <button
              className={`view-btn ${viewMode === 'pattern' ? 'active' : ''}`}
              onClick={() => setViewMode('pattern')}
            >
              PATTERN
            </button>
            <button
              className={`view-btn ${viewMode === 'song' ? 'active' : ''}`}
              onClick={() => setViewMode('song')}
            >
              SONG
            </button>
            <button
              className={`view-btn ${performMode ? 'active' : ''}`}
              onClick={togglePerformMode}
            >
              PERFORM
            </button>
          </div>

          <div className="track-strip-selector">
            {tracks.map((track) => (
              <TrackColumn key={track.id} trackId={track.id} />
            ))}
          </div>
        </aside>
      </div>

      <div className="encoder-section-wrapper">
        <EncoderSection />
      </div>

      <footer className="app-footer">
        <div className="status-indicator">
          <span className={`status-dot ${isConnected ? '' : 'disconnected'}`} />
          <span>{isConnected ? 'AUDIO READY' : 'INITIALIZING'}</span>
        </div>
        <div className="meter-display">
          <div className="meter-bar">
            <div
              className="meter-fill"
              style={{ width: `${Math.min(peakLevel * 100, 100)}%` }}
            />
          </div>
          <span className="meter-label">LEVEL</span>
        </div>
        <div className="latency-display">
          <span>{latency.toFixed(1)} ms</span>
        </div>
      </footer>
    </div>
  );
};

export default App;
