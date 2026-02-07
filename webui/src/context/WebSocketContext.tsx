import React, { createContext, useContext, useEffect, useRef, useCallback, useState } from 'react';
import { useTonverkStore } from '../tonverkStore';
import type { TransportState, Track } from '../tonverkStore';

const WS_URL = 'ws://localhost:8080/ws/audio';

interface AudioState {
  tracks: TrackState[];
  transport: TransportState;
}

interface TrackState {
  id: number;
  name: string;
  muted: boolean;
  solo: boolean;
  volume: number;
  pan: number;
  current_page: string;
  src_params: Record<string, number>;
  fltr_params: Record<string, number>;
  amp_params: Record<string, number>;
  fx_slots: FxSlotState[];
}

interface FxSlotState {
  id: number;
  fx_type: string;
  bypass: boolean;
  params: Record<string, number>;
}

type WebSocketMessage =
  | { type: 'SetTrackParam'; track: number; param: string; value: number }
  | { type: 'SetTrackMute'; track: number; muted: boolean }
  | { type: 'SetTrackSolo'; track: number; solo: boolean }
  | { type: 'SetTrackVolume'; track: number; volume: number }
  | { type: 'SetTrackPan'; track: number; pan: number }
  | { type: 'SetTempo'; tempo: number }
  | { type: 'Play' }
  | { type: 'Stop' };

interface WebSocketContextType {
  isConnected: boolean;
  sendMessage: (message: WebSocketMessage) => void;
  setTrackParam: (track: number, param: string, value: number) => void;
  setTrackMute: (track: number, muted: boolean) => void;
  setTrackSolo: (track: number, solo: boolean) => void;
  setTrackVolume: (track: number, volume: number) => void;
  setTrackPan: (track: number, pan: number) => void;
  setTempo: (tempo: number) => void;
  play: () => void;
  stop: () => void;
}

const WebSocketContext = createContext<WebSocketContextType | null>(null);

function mapBackendTrackToFrontend(backendTrack: TrackState): Partial<Track> {
  return {
    id: backendTrack.id,
    name: backendTrack.name,
    muted: backendTrack.muted,
    solo: backendTrack.solo,
    volume: backendTrack.volume,
    pan: backendTrack.pan,
    currentPage: backendTrack.current_page as 'trig' | 'src' | 'fltr' | 'amp' | 'fx' | 'mod',
    srcParams: backendTrack.src_params,
    fltrParams: backendTrack.fltr_params,
    ampParams: backendTrack.amp_params,
    fxSlots: backendTrack.fx_slots.map(slot => ({
      id: slot.id,
      type: slot.fx_type.toLowerCase(),
      bypass: slot.bypass,
      params: slot.params,
    })),
  };
}

export const WebSocketProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const isConnectingRef = useRef(false);
  const [isConnected, setIsConnected] = useState(false);

  const {
    setTransport,
    setTracks,
  } = useTonverkStore();

  const connect = useCallback(() => {
    if (isConnectingRef.current || (wsRef.current && wsRef.current.readyState === WebSocket.OPEN)) {
      return;
    }

    isConnectingRef.current = true;

    try {
      const ws = new WebSocket(WS_URL);
      wsRef.current = ws;

      ws.onopen = () => {
        console.log('[WebSocket] Connected to audio engine');
        isConnectingRef.current = false;
        setIsConnected(true);
      };

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);

          if (message.type === 'state_update') {
            const { tracks, transport } = message.state;

            setTracks(tracks.map(mapBackendTrackToFrontend));

            setTransport({
              playing: transport.playing,
              recording: transport.recording,
              tempo: transport.tempo,
              currentStep: transport.current_step,
              currentPage: transport.current_page,
              patternId: transport.pattern_id,
              patternBank: typeof transport.pattern_bank === 'number'
                ? transport.pattern_bank
                : transport.pattern_bank.charCodeAt(0) - 65,
              songPosition: 0,
              playMode: 'pattern',
            });
          }
        } catch (error) {
          console.error('[WebSocket] Failed to parse message:', error);
        }
      };

      ws.onerror = () => {
        isConnectingRef.current = false;
      };

      ws.onclose = () => {
        console.log('[WebSocket] Disconnected, attempting reconnect...');
        wsRef.current = null;
        isConnectingRef.current = false;
        setIsConnected(false);

        reconnectTimeoutRef.current = window.setTimeout(() => {
          connect();
        }, 2000);
      };
    } catch {
      isConnectingRef.current = false;
    }
  }, [setTransport, setTracks]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    setIsConnected(false);
  }, []);

  const sendMessage = useCallback((message: WebSocketMessage) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    }
  }, []);

  const setTrackParam = useCallback((track: number, param: string, value: number) => {
    sendMessage({ type: 'SetTrackParam', track, param, value });
  }, [sendMessage]);

  const setTrackMute = useCallback((track: number, muted: boolean) => {
    sendMessage({ type: 'SetTrackMute', track, muted });
  }, [sendMessage]);

  const setTrackSolo = useCallback((track: number, solo: boolean) => {
    sendMessage({ type: 'SetTrackSolo', track, solo });
  }, [sendMessage]);

  const setTrackVolume = useCallback((track: number, volume: number) => {
    sendMessage({ type: 'SetTrackVolume', track, volume });
  }, [sendMessage]);

  const setTrackPan = useCallback((track: number, pan: number) => {
    sendMessage({ type: 'SetTrackPan', track, pan });
  }, [sendMessage]);

  const setTempo = useCallback((tempo: number) => {
    sendMessage({ type: 'SetTempo', tempo });
  }, [sendMessage]);

  const play = useCallback(() => {
    sendMessage({ type: 'Play' });
  }, [sendMessage]);

  const stop = useCallback(() => {
    sendMessage({ type: 'Stop' });
  }, [sendMessage]);

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  return (
    <WebSocketContext.Provider value={{
      isConnected,
      sendMessage,
      setTrackParam,
      setTrackMute,
      setTrackSolo,
      setTrackVolume,
      setTrackPan,
      setTempo,
      play,
      stop,
    }}>
      {children}
    </WebSocketContext.Provider>
  );
}

export function useWebSocket() {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocket must be used within a WebSocketProvider');
  }
  return context;
}
