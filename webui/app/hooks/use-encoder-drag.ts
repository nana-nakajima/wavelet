import { useCallback, useRef, useEffect } from "react";

interface UseEncoderDragOptions {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  fine?: boolean;
}

const NORMAL_PX = 200;
const FINE_PX = 800;

export function useEncoderDrag({ value, onChange, min = 0, max = 1, fine = false }: UseEncoderDragOptions) {
  const dragging = useRef(false);
  const startY = useRef(0);
  const startVal = useRef(0);

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      dragging.current = true;
      startY.current = e.clientY;
      startVal.current = value;
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
      document.body.classList.add("encoder-active");
    },
    [value]
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!dragging.current) return;
      const range = max - min;
      const px = fine ? FINE_PX : NORMAL_PX;
      const delta = (startY.current - e.clientY) / px * range;
      const next = Math.max(min, Math.min(max, startVal.current + delta));
      onChange(next);
    },
    [onChange, min, max, fine]
  );

  const onPointerUp = useCallback(() => {
    dragging.current = false;
    document.body.classList.remove("encoder-active");
  }, []);

  useEffect(() => {
    return () => {
      document.body.classList.remove("encoder-active");
    };
  }, []);

  return { onPointerDown, onPointerMove, onPointerUp };
}
