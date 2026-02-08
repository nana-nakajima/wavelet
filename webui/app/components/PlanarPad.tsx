import { useCallback, useRef } from "react";
import { useWaveletStore } from "~/store/wavelet-store";

export function PlanarPad() {
  const planarX = useWaveletStore((s) => s.ui.planarX);
  const planarY = useWaveletStore((s) => s.ui.planarY);
  const setPlanar = useWaveletStore((s) => s.setPlanar);
  const padRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);

  const updateFromEvent = useCallback(
    (e: React.PointerEvent) => {
      const rect = padRef.current?.getBoundingClientRect();
      if (!rect) return;
      const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      const y = Math.max(0, Math.min(1, 1 - (e.clientY - rect.top) / rect.height));
      setPlanar(x, y);
    },
    [setPlanar]
  );

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      dragging.current = true;
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
      updateFromEvent(e);
    },
    [updateFromEvent]
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!dragging.current) return;
      updateFromEvent(e);
    },
    [updateFromEvent]
  );

  const onPointerUp = useCallback(() => {
    dragging.current = false;
  }, []);

  return (
    <div
      ref={padRef}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      className="relative w-[140px] h-[80px] bg-chassis-800 cursor-crosshair select-none m-2 rounded"
    >
      {/* Grid lines */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute left-1/2 top-0 bottom-0 w-px bg-chassis-700" />
        <div className="absolute top-1/2 left-0 right-0 h-px bg-chassis-700" />
      </div>

      {/* Dot */}
      <div
        className="absolute w-3 h-3 rounded-full bg-oled-teal shadow-[0_0_6px_rgba(0,229,200,0.4)] -translate-x-1/2 -translate-y-1/2 pointer-events-none"
        style={{
          left: `${planarX * 100}%`,
          top: `${(1 - planarY) * 100}%`,
        }}
      />

      {/* Axis labels */}
      <span className="absolute bottom-0.5 right-1 text-[6px] text-chassis-600">X</span>
      <span className="absolute top-0.5 left-1 text-[6px] text-chassis-600">Y</span>
    </div>
  );
}
