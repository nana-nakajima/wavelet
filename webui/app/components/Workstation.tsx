import { useState } from "react";
import { useWaveletStore } from "~/store/wavelet-store";
import { useEngineInit } from "~/engine/use-engine-init";
import { useEngineSync } from "~/engine/use-engine-sync";
import { useKeyboardShortcuts } from "~/hooks/use-keyboard-shortcuts";
import { TransportBar } from "./TransportBar";
import { OledScreen } from "./OledScreen";
import { FxSlotPanel } from "./FxSlotPanel";
import { PlanarPad } from "./PlanarPad";
import { PageButtons } from "./PageButtons";
import { FuncKey } from "./FuncKey";
import { StepGrid } from "./StepGrid";
import { PianoKeyboard } from "./PianoKeyboard";
import { EncoderSection } from "./EncoderSection";
import { MasterSection } from "./MasterSection";
import { TrackSelector } from "./TrackSelector";
import { TrackStrips } from "./TrackStrips";
import { StatusBar } from "./StatusBar";

export function Workstation() {
  const engineReady = useWaveletStore((s) => s.engineReady);
  const { initEngine } = useEngineInit();
  const [initError, setInitError] = useState<string | null>(null);
  const [initializing, setInitializing] = useState(false);
  useEngineSync();
  useKeyboardShortcuts();

  const handleInit = async () => {
    setInitError(null);
    setInitializing(true);
    try {
      await initEngine();
    } catch (err) {
      setInitError(String(err));
    } finally {
      setInitializing(false);
    }
  };

  if (!engineReady) {
    return (
      <div
        className="flex h-screen w-screen flex-col items-center justify-center bg-chassis-950 cursor-pointer"
        onClick={handleInit}
        onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") handleInit(); }}
        role="button"
        tabIndex={0}
      >
        <div className="flex flex-col items-center gap-4">
          <h1 className="text-2xl font-display font-bold tracking-widest text-oled-teal">
            WAVELET
          </h1>
          {initializing ? (
            <p className="text-xs text-oled-amber tracking-wider uppercase">
              Initializing engine...
            </p>
          ) : initError ? (
            <div className="flex flex-col items-center gap-2">
              <p className="text-xs text-oled-red tracking-wider uppercase">
                Engine init failed
              </p>
              <p className="text-[10px] text-chassis-600 max-w-xs text-center">
                {initError}
              </p>
              <p className="text-xs text-chassis-600 tracking-wider uppercase mt-2">
                Click to retry
              </p>
            </div>
          ) : (
            <p className="text-xs text-chassis-600 tracking-wider uppercase animate-pulse">
              Click to initialize audio engine
            </p>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen w-screen flex-col bg-chassis-950 font-mono overflow-hidden">
      {/* Top: Transport */}
      <TransportBar />

      {/* Middle: main content area */}
      <div className="flex flex-1 min-h-0">
        {/* Left sidebar: Track Selector */}
        <TrackSelector />

        {/* Center panel */}
        <div className="flex flex-1 flex-col min-w-0">
          {/* OLED Screen */}
          <div className="flex-shrink-0 border-b border-chassis-700">
            <OledScreen />
          </div>

          {/* FX + Planar row */}
          <div className="flex flex-shrink-0 border-b border-chassis-700">
            <div className="flex-1 border-r border-chassis-700">
              <FxSlotPanel />
            </div>
            <div className="flex-shrink-0">
              <PlanarPad />
            </div>
          </div>

          {/* Page buttons + FUNC */}
          <div className="flex items-center gap-2 px-3 py-1.5 flex-shrink-0 border-b border-chassis-700">
            <PageButtons />
            <div className="ml-auto">
              <FuncKey />
            </div>
          </div>

          {/* Step Grid */}
          <div className="flex-shrink-0 border-b border-chassis-700">
            <StepGrid />
          </div>

          {/* Piano Keyboard */}
          <div className="flex-1 min-h-0">
            <PianoKeyboard />
          </div>
        </div>

        {/* Right panel: Track Strips */}
        <TrackStrips />
      </div>

      {/* Bottom: Master + Encoders */}
      <div className="flex items-center gap-4 px-3 py-2 border-t border-chassis-700 bg-chassis-900 flex-shrink-0">
        <MasterSection />
        <div className="h-8 w-px bg-chassis-700" />
        <EncoderSection />
      </div>

      {/* Status Bar */}
      <StatusBar />
    </div>
  );
}
