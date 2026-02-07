import React from 'react';
import { useStore } from '../store';

interface PortProps {
  id: string;
  label: string;
  type: 'input' | 'output';
  color: string;
  moduleId: string;
}

export function Port({ id, label, type, color, moduleId }: PortProps) {
  const { setHoveredPort, hoveredPort } = useStore();

  const handleMouseEnter = () => {
    setHoveredPort({ moduleId, port: id, type });
  };

  const handleMouseLeave = () => {
    setHoveredPort(null);
  };

  const handleClick = () => {
    // TODO: Implement cable connection state machine
    // Start connection on output port click, complete on input port click
  };

  const isHovered = hoveredPort?.moduleId === moduleId && hoveredPort?.port === id;

  return (
    <div
      className="relative flex items-center gap-1 group"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onClick={handleClick}
    >
      {/* Port - Soul orb style */}
      <div
        className={`port ${type} transition-all duration-150 ${
          isHovered ? 'scale-125' : ''
        }`}
        style={{
          backgroundColor: isHovered ? color : '#0f1014',
          borderColor: isHovered ? color : '#3a3a4a',
          boxShadow: isHovered
            ? `0 0 12px ${color}, 0 0 20px ${color}44, inset 0 0 6px ${color}88`
            : 'inset 1px 1px 2px rgba(0, 0, 0, 0.5)',
        }}
      >
        {/* Inner glow */}
        <div
          className="absolute inset-1 rounded-full transition-opacity"
          style={{
            background: `radial-gradient(circle, ${color}44 0%, transparent 70%)`,
            opacity: isHovered ? 1 : 0,
          }}
        />

        {/* Center dot */}
        {!isHovered && (
          <div
            className="absolute top-1/2 left-1/2 w-1 h-1 rounded-full -translate-x-1/2 -translate-y-1/2"
            style={{
              backgroundColor: color,
              opacity: 0.4,
            }}
          />
        )}
      </div>

      {/* Cable preview on hover */}
      {isHovered && type === 'output' && (
        <div
          className="absolute pointer-events-none animate-pulse"
          style={{
            left: '100%',
            top: '50%',
            width: '30px',
            height: '2px',
            background: `linear-gradient(90deg, ${color}, transparent)`,
            boxShadow: `0 0 8px ${color}`,
          }}
        />
      )}

      {/* Label */}
      <span
        className={`text-[8px] uppercase tracking-wider transition-all duration-150 ${
          isHovered ? 'text-white' : 'text-[#484858]'
        }`}
        style={{
          textShadow: isHovered ? `0 0 8px ${color}` : 'none',
        }}
      >
        {label}
      </span>
    </div>
  );
}
