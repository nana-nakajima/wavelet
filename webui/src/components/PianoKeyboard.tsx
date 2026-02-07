import React, { useState, useCallback } from 'react';

interface PianoKeyProps {
  note: number;
  label: string;
  isBlack: boolean;
  active: boolean;
  onNoteOn: (note: number) => void;
  onNoteOff: (note: number) => void;
}

const PianoKey: React.FC<PianoKeyProps> = ({ note, label, isBlack, active, onNoteOn, onNoteOff }) => {
  const [isPressed, setIsPressed] = useState(false);

  const handleMouseDown = useCallback(() => {
    setIsPressed(true);
    onNoteOn(note);
  }, [note, onNoteOn]);

  const handleMouseUp = useCallback(() => {
    setIsPressed(false);
    onNoteOff(note);
  }, [note, onNoteOff]);

  const handleMouseLeave = useCallback(() => {
    if (isPressed) {
      setIsPressed(false);
      onNoteOff(note);
    }
  }, [isPressed, note, onNoteOff]);

  return (
    <div
      className={`piano-key ${isBlack ? 'black' : 'white'} ${active || isPressed ? 'active' : ''}`}
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseLeave}
    >
      {!isBlack && <span className="key-label">{label}</span>}
    </div>
  );
};

export const PianoKeyboard: React.FC = () => {
  const [activeNotes, setActiveNotes] = useState<Set<number>>(new Set());
  const tracks = useTonverkStore(state => state.tracks);
  const selectedTrackId = useTonverkStore(state => state.selectedTrackId);

  const track = tracks.find(t => t.id === selectedTrackId) || tracks[0];

  const handleNoteOn = useCallback((note: number) => {
    setActiveNotes(prev => new Set([...prev, note]));
    console.log(`Note ON: ${note} on track ${track.id}`);
  }, [track.id]);

  const handleNoteOff = useCallback((note: number) => {
    setActiveNotes(prev => {
      const next = new Set(prev);
      next.delete(note);
      return next;
    });
    console.log(`Note OFF: ${note}`);
  }, []);

  const notes = [
    { note: 60, label: 'C', isBlack: false },
    { note: 61, label: 'C#', isBlack: true },
    { note: 62, label: 'D', isBlack: false },
    { note: 63, label: 'D#', isBlack: true },
    { note: 64, label: 'E', isBlack: false },
    { note: 65, label: 'F', isBlack: false },
    { note: 66, label: 'F#', isBlack: true },
    { note: 67, label: 'G', isBlack: false },
    { note: 68, label: 'G#', isBlack: true },
    { note: 69, label: 'A', isBlack: false },
    { note: 70, label: 'A#', isBlack: true },
    { note: 71, label: 'B', isBlack: false },
    { note: 72, label: 'C', isBlack: false },
    { note: 73, label: 'C#', isBlack: true },
    { note: 74, label: 'D', isBlack: false },
    { note: 75, label: 'D#', isBlack: true },
    { note: 76, label: 'E', isBlack: false },
    { note: 77, label: 'F', isBlack: false },
    { note: 78, label: 'F#', isBlack: true },
    { note: 79, label: 'G', isBlack: false },
    { note: 80, label: 'G#', isBlack: true },
    { note: 81, label: 'A', isBlack: false },
    { note: 82, label: 'A#', isBlack: true },
    { note: 83, label: 'B', isBlack: false },
  ];

  return (
    <div className="piano-keyboard">
      <div className="keyboard-header">KEYBOARD</div>
      <div className="keyboard-octave">
        <div className="keyboard-keys">
          {notes.map(({ note, label, isBlack }) => (
            <PianoKey
              key={note}
              note={note}
              label={label}
              isBlack={isBlack}
              active={activeNotes.has(note)}
              onNoteOn={handleNoteOn}
              onNoteOff={handleNoteOff}
            />
          ))}
        </div>
      </div>
      <div className="keyboard-velocity">
        <span>VEL: 100</span>
      </div>
    </div>
  );
};

// Import hook for piano keyboard
import { useTonverkStore } from '../tonverkStore';
