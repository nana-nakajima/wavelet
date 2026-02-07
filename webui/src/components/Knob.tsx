import React, { useRef, useState, useEffect, useCallback } from 'react';

interface KnobProps {
  label: string;
  value: number;
  color: string;
  size?: number;
  min?: number;
  max?: number;
  onChange: (value: number) => void;
}

export function Knob({
  label,
  value,
  color,
  size = 36,
  min = 0,
  max = 1,
  onChange
}: KnobProps) {
  const knobRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);

  // Use refs to avoid stale closures in event handlers
  const dragState = useRef({ startY: 0, startValue: 0 });
  const propsRef = useRef({ min, max, onChange });
  propsRef.current = { min, max, onChange };

  const rotation = -135 + (value - min) / (max - min) * 270;

  const handleMouseMove = useCallback((e: MouseEvent) => {
    const { startY, startValue } = dragState.current;
    const { min, max, onChange } = propsRef.current;

    const deltaY = startY - e.clientY;
    const range = max - min;
    const sensitivity = range / 100;

    let newValue = startValue + deltaY * sensitivity;
    newValue = Math.max(min, Math.min(max, newValue));

    onChange(newValue);
  }, []);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  }, [handleMouseMove]);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    dragState.current = { startY: e.clientY, startValue: value };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  const handleTouchMove = useCallback((e: TouchEvent) => {
    const { startY, startValue } = dragState.current;
    const { min, max, onChange } = propsRef.current;

    const deltaY = startY - e.touches[0].clientY;
    const range = max - min;
    const sensitivity = range / 100;

    let newValue = startValue + deltaY * sensitivity;
    newValue = Math.max(min, Math.min(max, newValue));

    onChange(newValue);
  }, []);

  const handleTouchEnd = useCallback(() => {
    setIsDragging(false);
    document.removeEventListener('touchmove', handleTouchMove);
    document.removeEventListener('touchend', handleTouchEnd);
  }, [handleTouchMove]);

  const handleTouchStart = (e: React.TouchEvent) => {
    setIsDragging(true);
    dragState.current = { startY: e.touches[0].clientY, startValue: value };

    document.addEventListener('touchmove', handleTouchMove);
    document.addEventListener('touchend', handleTouchEnd);
  };

  // Double click to reset
  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onChange((min + max) / 2);
  };

  useEffect(() => {
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.removeEventListener('touchmove', handleTouchMove);
      document.removeEventListener('touchend', handleTouchEnd);
    };
  }, [handleMouseMove, handleMouseUp, handleTouchMove, handleTouchEnd]);

  return (
    <div className="flex flex-col items-center gap-1">
      <div
        ref={knobRef}
        className="relative cursor-ns-resize select-none group"
        style={{ width: size, height: size }}
        onMouseDown={handleMouseDown}
        onTouchStart={handleTouchStart}
        onDoubleClick={handleDoubleClick}
      >
        {/* Outer glow ring */}
        <div
          className="absolute inset-[-2px] rounded-full opacity-0 group-hover:opacity-100 transition-opacity"
          style={{
            background: `radial-gradient(circle, ${color}33 0%, transparent 70%)`,
          }}
        />

        {/* Knob Background - Dead Cells artifact style */}
        <div
          className="absolute inset-0 rounded-full border-2 border-[#252530] transition-all group-hover:border-opacity-50"
          style={{
            background: `
              radial-gradient(circle at 30% 30%, #22262f 0%, #0f1014 100%),
              conic-gradient(
                from ${rotation - 135}deg,
                ${color} 0deg,
                ${color}44 45deg,
                #0f1014 90deg,
                #0f1014 270deg,
                ${color}44 315deg,
                ${color} 360deg
              )
            `,
            backgroundBlendMode: 'overlay',
            boxShadow: isDragging
              ? `inset 0 2px 4px rgba(0, 0, 0, 0.6), 0 0 15px ${color}55`
              : `inset 0 2px 4px rgba(0, 0, 0, 0.5), 0 2px 4px rgba(0, 0, 0, 0.3)`,
          }}
        >
          {/* Arc indicator */}
          <svg className="absolute inset-0 w-full h-full" viewBox="0 0 100 100">
            <circle
              cx="50"
              cy="50"
              r="42"
              fill="none"
              stroke={color}
              strokeWidth="3"
              strokeLinecap="round"
              strokeDasharray={`${((value - min) / (max - min)) * 200} 1000`}
              strokeDashoffset="-50"
              style={{
                filter: `drop-shadow(0 0 4px ${color})`,
                transform: 'rotate(-135deg)',
                transformOrigin: 'center',
              }}
            />
          </svg>

          {/* Center indicator line */}
          <div
            className="absolute w-0.5 h-3 bg-white rounded-sm left-1/2"
            style={{
              transform: `translateX(-50%) rotate(${rotation}deg)`,
              transformOrigin: '50% 300%',
              top: '8%',
              boxShadow: `0 0 4px ${color}`,
            }}
          />

          {/* Center dot */}
          <div
            className="absolute top-1/2 left-1/2 w-2 h-2 rounded-full -translate-x-1/2 -translate-y-1/2"
            style={{
              backgroundColor: color,
              boxShadow: `0 0 6px ${color}`,
              opacity: 0.6,
            }}
          />
        </div>

        {/* Value display on drag */}
        {isDragging && (
          <div
            className="absolute -top-7 left-1/2 -translate-x-1/2 bg-[#0a0a0c] px-2 py-0.5 rounded-sm text-[10px] whitespace-nowrap border border-[#252530] z-50"
            style={{
              color,
              boxShadow: `0 0 10px ${color}33`,
            }}
          >
            {value.toFixed(2)}
          </div>
        )}
      </div>

      {/* Label */}
      <span className="text-[9px] text-[#686878] uppercase tracking-wider text-center truncate max-w-full">
        {label}
      </span>
    </div>
  );
}
