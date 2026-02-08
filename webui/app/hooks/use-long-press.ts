import { useCallback, useRef } from "react";

const LONG_PRESS_MS = 300;

export function useLongPress(
  onLongPress: () => void,
  onRelease: () => void,
  onClick?: () => void
) {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const longPressed = useRef(false);

  const onPointerDown = useCallback(() => {
    longPressed.current = false;
    timer.current = setTimeout(() => {
      longPressed.current = true;
      onLongPress();
    }, LONG_PRESS_MS);
  }, [onLongPress]);

  const onPointerUp = useCallback(() => {
    if (timer.current) {
      clearTimeout(timer.current);
      timer.current = null;
    }
    if (longPressed.current) {
      onRelease();
    } else {
      onClick?.();
    }
    longPressed.current = false;
  }, [onRelease, onClick]);

  const onPointerLeave = useCallback(() => {
    if (timer.current) {
      clearTimeout(timer.current);
      timer.current = null;
    }
    if (longPressed.current) {
      onRelease();
      longPressed.current = false;
    }
  }, [onRelease]);

  return { onPointerDown, onPointerUp, onPointerLeave };
}
