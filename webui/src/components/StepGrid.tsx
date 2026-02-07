import React from 'react';
import { useTonverkStore, TRACK_TYPE_CONFIG } from '../tonverkStore';

interface StepButtonProps {
  active: boolean;
  current: boolean;
  color: string;
  onClick: () => void;
}

const StepButton: React.FC<StepButtonProps> = ({ active, current, color, onClick }) => {
  return (
    <button
      className={`step-button ${active ? 'active' : ''} ${current ? 'current' : ''}`}
      onClick={onClick}
      style={{
        width: 24,
        height: 24,
        borderRadius: '50%',
        backgroundColor: active ? color : '#1a1a1a',
        boxShadow: active && current ? `0 0 12px ${color}, inset 0 0 4px ${color}` : active ? `0 0 8px ${color}` : 'none',
        border: current ? `2px solid ${color}` : '1px solid #333',
        transition: 'all 0.1s ease',
      }}
    />
  );
};

export const StepGrid: React.FC = () => {
  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);
  const transport = useTonverkStore(state => state.transport);
  const setStepTrigType = useTonverkStore(state => state.setStepTrigType);
  const viewMode = useTonverkStore(state => state.viewMode);

  const track = tracks.find(t => t.id === selectedTrackId) || tracks[0];
  const config = TRACK_TYPE_CONFIG[track.type];
  const currentStepIndex = transport.currentStep % 16;
  const currentPage = track.sequencer.pages[transport.currentPage];

  const handleStepClick = (stepIndex: number) => {
    const step = currentPage.steps[stepIndex];
    const types: Array<'none' | 'note' | 'lock' | 'combined'> = ['none', 'note', 'lock', 'combined'];
    const currentIndex = types.indexOf(step.trigType);
    const nextIndex = (currentIndex + 1) % types.length;
    setStepTrigType(track.id, transport.currentPage, stepIndex, types[nextIndex]);
  };

  if (viewMode === 'song') {
    return null;
  }

  return (
    <div className="step-grid-global">
      <div className="step-grid-label">STEP</div>
      <div className="step-buttons-row">
        {Array.from({ length: 16 }, (_, i) => {
          const step = currentPage.steps[i];
          return (
            <StepButton
              key={i}
              active={step.trigType !== 'none'}
              current={i === currentStepIndex && transport.playing}
              color={config.color}
              onClick={() => handleStepClick(i)}
            />
          );
        })}
      </div>
      <div className="step-page-indicator">
        Page {transport.currentPage + 1}/16
      </div>
    </div>
  );
};
