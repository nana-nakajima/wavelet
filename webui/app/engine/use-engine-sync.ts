// ==========================================================================
// use-engine-sync.ts — RAF loop that reads engine state into the Zustand store
// ==========================================================================

import { useEffect, useRef } from "react";
import { useWaveletStore } from "~/store/wavelet-store";

/**
 * Starts a requestAnimationFrame loop that reads engine-computed values
 * (step position, peak levels, CPU load, waveform) from the SharedArrayBuffer
 * and writes them into the Zustand store for UI rendering.
 *
 * Call this once at the top level of your client-only component tree.
 */
export function useEngineSync() {
  const syncFromEngine = useWaveletStore((s) => s.syncFromEngine);
  const engineReady = useWaveletStore((s) => s.engineReady);
  const rafRef = useRef<number>(0);

  useEffect(() => {
    if (!engineReady) return;

    function tick() {
      syncFromEngine();
      rafRef.current = requestAnimationFrame(tick);
    }

    rafRef.current = requestAnimationFrame(tick);

    return () => {
      cancelAnimationFrame(rafRef.current);
    };
  }, [engineReady, syncFromEngine]);
}
