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
  const { setHoveredPort, addConnection, hoveredPort } = useStore();

  const handleMouseEnter = () => {
    setHoveredPort({ moduleId, port: id, type });
  };

  const handleMouseLeave = () => {
    setHoveredPort(null);
  };

  const handleClick = () => {
    // This would start/end cable connections
    console.log(`Clicked ${type} port: ${label}`);
  };

  const isHovered = hoveredPort?.moduleId === moduleId && hoveredPort?.port === id;

  return (
    <div 
      className="relative flex items-center gap-1 group"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onClick={handleClick}
    >
      {/* Port */}
      <div
        className={`port ${type} transition-all ${
          isHovered ? 'scale-125 ring-2 ring-white' : ''
        }`}
        style={{
          backgroundColor: isHovered ? color : '#2d2d2d',
          borderColor: color,
        }}
      >
        {/* Inner highlight */}
        <div 
          className="absolute inset-0 rounded-full opacity-30"
          style={{ background: `radial-gradient(circle, ${color} 0%, transparent 70%)` }}
        />
      </div>
      
      {/* Cable preview on hover */}
      {isHovered && type === 'output' && (
        <div 
          className="absolute pointer-events-none"
          style={{
            left: '100%',
            top: '50%',
            width: '20px',
            height: '2px',
            background: `linear-gradient(90deg, ${color}, transparent)`,
          }}
        />
      )}

      {/* Label */}
      <span 
        className={`text-[8px] uppercase tracking-wide transition-colors ${
          isHovered ? 'text-white' : 'text-[#737373]'
        }`}
      >
        {label}
      </span>
    </div>
  );
}
