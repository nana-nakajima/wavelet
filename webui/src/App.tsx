import React from 'react';
import { TrackColumn, TransportBar, TrackSelector } from './components/TrackColumn';
import { useTonverkStore } from './tonverkStore';

const App: React.FC = () => {
  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const showBrowser = useTonverkStore(state => state.showBrowser);
  const setSelectedTrack = useTonverkStore(state => state.setSelectedTrack);

  return (
    <div className="wavelet-app">
      <header className="app-header">
        <h1>WAVELET</h1>
        <span className="version">Tonverk Edition</span>
      </header>

      <TransportBar />

      <div className="main-content">
        <TrackSelector />

        <div className="track-display">
          <div className="tracks-container">
            {tracks.map((track) => (
              <TrackColumn key={track.id} trackId={track.id} />
            ))}
          </div>
        </div>

        {showBrowser && (
          <div className="browser-panel">
            <h3>Sample Browser</h3>
            <p>Select a track to browse samples</p>
          </div>
        )}
      </div>

      <footer className="app-footer">
        <div className="status-indicator">
          <span className="status-dot" />
          <span>Connected</span>
        </div>
        <div className="cpu-meter">
          <span>CPU: 23%</span>
        </div>
      </footer>
    </div>
  );
};

export default App;
