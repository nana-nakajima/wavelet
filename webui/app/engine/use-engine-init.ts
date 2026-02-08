// ==========================================================================
// use-engine-init.ts — Hook to initialize the WASM audio engine
//
// Must be called inside a user-gesture handler (click/keydown) because
// AudioContext requires a user gesture to start.
// ==========================================================================

import { useCallback, useRef } from "react";
import { engine } from "~/engine/wavelet-engine";
import { useWaveletStore } from "~/store/wavelet-store";

/**
 * Returns an `initEngine` callback that:
 *   1. Loads the WASM binary
 *   2. Creates the AudioContext + AudioWorklet
 *   3. Attaches the SharedArrayBuffer
 *   4. Resumes audio
 *   5. Marks the store as ready
 *
 * Safe to call multiple times — subsequent calls are no-ops.
 */
export function useEngineInit(wasmUrl = "/wavelet_bg.wasm") {
  const setEngineReady = useWaveletStore((s) => s.setEngineReady);
  const initedRef = useRef(false);

  const initEngine = useCallback(async () => {
    if (initedRef.current) return;
    initedRef.current = true;

    try {
      // Attach event listeners BEFORE init so we don't miss the "ready" event
      engine.on("ready", () => {
        setEngineReady(true);
      });

      engine.on("error", (msg) => {
        console.error("[WaveletEngine]", msg);
      });

      await engine.init(wasmUrl);
      await engine.resume();
    } catch (err) {
      console.error("[WaveletEngine] Init failed:", err);
      initedRef.current = false; // Allow retry
      throw err; // Propagate so callers can show error UI
    }
  }, [wasmUrl, setEngineReady]);

  return { initEngine, engine };
}
