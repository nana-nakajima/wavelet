import React, { createContext, useContext, useEffect, useRef, useCallback, useState } from 'react';
import { useTonverkStore } from '../tonverkStore';
import { audioEngine } from '../audio/engine';
import { midiHandler } from '../audio/midi';

interface AudioContextType {
  isReady: boolean;
  isPlaying: boolean;
  isRecording: boolean;
  isConnected: boolean;
  latency: number;
  peakLevel: number;
  rmsLevel: number;
  init: () => Promise<void>;
  resume: () => Promise<void>;
  play: () => void;
  stop: () => void;
  setTempo: (bpm: number) => void;
  setMasterVolume: (volume: number) => void;
  setTrackParam: (track: number, param: string, value: number) => void;
  setTrackVolume: (track: number, volume: number) => void;
  setTrackMute: (track: number, muted: boolean) => void;
  setTrackSolo: (track: number, solo: boolean) => void;
  noteOn: (note: number, velocity: number, track?: number) => void;
  noteOff: (note: number, track?: number) => void;
  midiReady: boolean;
  midiDevices: { id: string; name: string; state: string }[];
}

const AudioContext = createContext<AudioContextType | null>(null);

export const AudioProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [isReady, setIsReady] = useState(false);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isRecording, setIsRecording] = useState(false);
  const [isConnected, setIsConnected] = useState(false);
  const [latency, setLatency] = useState(0);
  const [peakLevel, setPeakLevel] = useState(0);
  const [rmsLevel, setRmsLevel] = useState(0);
  const [midiReady, setMidiReady] = useState(false);
  const [midiDevices, setMidiDevices] = useState<{ id: string; name: string; state: string }[]>([]);

  const animationFrameRef = useRef<number | null>(null);
  const {
    setTransport,
    updateTrackParam,
    setTracks,
    toggleMute,
    toggleSolo,
  } = useTonverkStore();

  const updateMeters = useCallback(() => {
    if (isReady) {
      setPeakLevel(audioEngine.getPeakLevel());
      setRmsLevel(audioEngine.getRmsLevel());
      setLatency(audioEngine.getLatency() * 1000);
    }
    animationFrameRef.current = requestAnimationFrame(updateMeters);
  }, [isReady]);

  useEffect(() => {
    animationFrameRef.current = requestAnimationFrame(updateMeters);
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [updateMeters]);

  const init = useCallback(async () => {
    if (isReady) return;

    try {
      await audioEngine.init();
      setIsReady(true);
      setIsConnected(true);

      // Initialize MIDI
      const midiOk = await midiHandler.init();
      setMidiReady(midiOk);

      if (midiOk) {
        midiHandler.subscribe((msg) => {
          if (msg.type === 'noteOn' && msg.velocity && msg.velocity > 0) {
            audioEngine.noteOn(msg.note!, msg.velocity * 127, msg.channel - 1);
          } else if (msg.type === 'noteOff') {
            audioEngine.noteOff(msg.note!, msg.channel - 1);
          } else if (msg.type === 'cc' && msg.controller !== undefined) {
            // Handle CC messages
            console.log('CC:', msg.controller, msg.value);
          }
        });

        midiHandler.getDevices();
      }

      console.log('Audio engine ready');
    } catch (error) {
      console.error('Failed to initialize audio:', error);
    }
  }, [isReady]);

  const resume = useCallback(async () => {
    await audioEngine.resume();
  }, []);

  const play = useCallback(() => {
    audioEngine.play();
    setIsPlaying(true);
    setTransport({ playing: true });
  }, [setTransport]);

  const stop = useCallback(() => {
    audioEngine.stop();
    setIsPlaying(false);
    setTransport({ playing: false, currentStep: 0 });
  }, [setTransport]);

  const setTempo = useCallback((bpm: number) => {
    audioEngine.setTempo(bpm);
    setTransport({ tempo: bpm });
  }, [setTransport]);

  const setMasterVolume = useCallback((volume: number) => {
    audioEngine.setMasterVolume(volume);
  }, []);

  const setTrackParam = useCallback((track: number, param: string, value: number) => {
    audioEngine.setTrackParam(track, param, value);
    updateTrackParam(track, param, value);
  }, [updateTrackParam]);

  const setTrackVolume = useCallback((track: number, volume: number) => {
    audioEngine.setTrackVolume(track, volume / 127);
  }, []);

  const setTrackMute = useCallback((track: number, muted: boolean) => {
    audioEngine.setTrackMute(track, muted);
    toggleMute(track);
  }, [toggleMute]);

  const setTrackSolo = useCallback((track: number, solo: boolean) => {
    audioEngine.setTrackSolo(track, solo);
    toggleSolo(track);
  }, [toggleSolo]);

  const noteOn = useCallback((note: number, velocity: number, track: number = 0) => {
    audioEngine.noteOn(note, velocity, track);
  }, []);

  const noteOff = useCallback((note: number, track: number = 0) => {
    audioEngine.noteOff(note, track);
  }, []);

  return (
    <AudioContext.Provider value={{
      isReady,
      isPlaying,
      isRecording,
      isConnected,
      latency,
      peakLevel,
      rmsLevel,
      init,
      resume,
      play,
      stop,
      setTempo,
      setMasterVolume,
      setTrackParam,
      setTrackVolume,
      setTrackMute,
      setTrackSolo,
      noteOn,
      noteOff,
      midiReady,
      midiDevices,
    }}>
      {children}
    </AudioContext.Provider>
  );
}

export function useAudio() {
  const context = useContext(AudioContext);
  if (!context) {
    throw new Error('useAudio must be used within an AudioProvider');
  }
  return context;
}
