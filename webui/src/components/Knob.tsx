import React, { useRef, useState, useEffect } from 'react';

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
  const [startY, setStartY] = useState(0);
  const [startValue, setStartValue] = useState(0);

  const rotation = -135 + (value - min) / (max - min) * 270;

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    setStartY(e.clientY);
    setStartValue(value);
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging) return;
    
    const deltaY = startY - e.clientY;
    const range = max - min;
    const sensitivity = range / 100; // 100px for full range
    
    let newValue = startValue + deltaY * sensitivity;
    newValue = Math.max(min, Math.min(max, newValue));
    
    onChange(newValue);
  };

  const handleMouseUp = () => {
    setIsDragging(false);
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  };

  // Touch support
  const handleTouchStart = (e: React.TouchEvent) => {
    setIsDragging(true);
    setStartY(e.touches[0].clientY);
    setStartValue(value);
    
    document.addEventListener('touchmove', handleTouchMove);
    document.addEventListener('touchend', handleTouchEnd);
  };

  const handleTouchMove = (e: TouchEvent) => {
    if (!isDragging) return;
    
    const deltaY = startY - e.touches[0].clientY;
    const range = max - min;
    const sensitivity = range / 100;
    
    let newValue = startValue + deltaY * sensitivity;
    newValue = Math.max(min, Math.min(max, newValue));
    
    onChange(newValue);
  };

  const handleTouchEnd = () => {
    setIsDragging(false);
    document.removeEventListener('touchmove', handleTouchMove);
    document.removeEventListener('touchend', handleTouchEnd);
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
  }, []);

  return (
    <div className="flex flex-col items-center gap-1">
      <div
        ref={knobRef}
        className="relative cursor-ns-resize select-none"
        style={{ width: size, height: size }}
        onMouseDown={handleMouseDown}
        onTouchStart={handleTouchStart}
        onDoubleClick={handleDoubleClick}
      >
        {/* Knob Background */}
        <div 
          className="absolute inset-0 rounded-full"
          style={{
            background: `conic-gradient(
              ${color} ${rotation}deg,
              #2d2d2d ${rotation}deg,
              #2d2d2d ${rotation + 270}deg,
              ${color} ${rotation + 270}deg
            )`,
            boxShadow: `
              inset 0 2px 4px rgba(0, 0, 0, 0.5),
              0 2px 4px rgba(0, 0, 0, 0.3)
            `,
          }}
        >
          {/* Center indicator */}
          <div 
            className="absolute top-1 left-1/2 -translate-x-1/2 w-1 h-2 bg-white rounded-sm"
            style={{
              transform: `translate(-50%, 2px) rotate(${rotation}deg)`,
              transformOrigin: '50% 100%',
              top: '15%',
            }}
          />
        </div>
        
        {/* Value display on hover/drag */}
        {isDragging && (
          <div 
            className="absolute -top-6 left-1/2 -translate-x-1/2 bg-[#1a1a1a] px-2 py-0.5 rounded text-[10px] whitespace-nowrap border border-[#404040]"
            style={{ color }}
          >
            {value.toFixed(2)}
          </div>
        )}
      </div>
      
      {/* Label */}
      <span className="text-[9px] text-[#a3a3a3] uppercase tracking-wide text-center max-w-[size] truncate">
        {label}
      </span>
    </div>
  );
}
