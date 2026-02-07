import React from 'react';
import { useStore, MODULE_SPECS } from '../store';
import { ModulePanel } from './ModulePanel';
import { Icons } from './Icons';

export function Rack() {
  const { modules, connections, removeModule } = useStore();

  // Calculate rack height based on modules
  const rackHeight = Math.max(800, modules.length * 120 + 200);

  return (
    <div className="flex-1 flex flex-col overflow-hidden bg-[#0a0a0c]">
      {/* Rack Header */}
      <div className="h-10 bg-[#0f1014] border-b-2 border-[#252530] flex items-center px-4 relative">
        {/* Decorative line */}
        <div className="absolute bottom-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-[#3a3a4a] to-transparent" />

        <div className="flex-1 flex items-center gap-4">
          <span className="text-xs text-[#484858] uppercase tracking-wider">Rack</span>
          <span className="text-xs text-[#00ff88]">
            {modules.length} module{modules.length !== 1 ? 's' : ''}
          </span>
          {modules.length === 0 && (
            <span className="text-xs text-[#484858] italic">
              Add modules from the browser
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => modules.forEach(m => removeModule(m.id))}
            className="pixel-button px-2 py-1 text-xs flex items-center gap-1 text-[#ff3366] hover:text-[#ff6688] disabled:opacity-30"
            disabled={modules.length === 0}
          >
            <Icons.Trash2 />
            Clear All
          </button>
        </div>
      </div>

      {/* Rack Mount */}
      <div className="flex-1 overflow-auto relative">
        {/* Rack Rails - Dungeon wall style */}
        <div
          className="min-h-full relative"
          style={{
            background: `
              linear-gradient(180deg, #0a0a0c 0%, #0f1014 50%, #0a0a0c 100%),
              repeating-linear-gradient(
                0deg,
                transparent,
                transparent 79px,
                #1a1d26 79px,
                #1a1d26 80px
              )
            `,
          }}
        >
          {/* Left Rail */}
          <div className="absolute left-0 top-0 bottom-0 w-4 bg-gradient-to-r from-[#1a1d26] to-[#0f1014] border-r border-[#252530]">
            {/* Mount holes with glow */}
            {Array.from({ length: Math.ceil(rackHeight / 80) }).map((_, i) => (
              <div
                key={i}
                className="rack-hole mx-auto my-4"
                style={{
                  boxShadow: i % 3 === 0 ? '0 0 4px #00ff8822' : 'inset 1px 1px 2px rgba(0,0,0,0.8)',
                }}
              />
            ))}
          </div>

          {/* Modules Container */}
          <div className="absolute left-4 right-4 top-0">
            {/* Rails overlay with glow */}
            <div className="absolute inset-y-0 left-0 right-0 border-l-2 border-r-2 border-[#252530] pointer-events-none">
              <div className="absolute inset-0 bg-gradient-to-r from-[#00ff8808] via-transparent to-[#bf5fff08]" />
            </div>

            {/* Cable Layer */}
            <svg className="absolute inset-0 w-full h-full pointer-events-none z-0">
              <defs>
                <filter id="glow">
                  <feGaussianBlur stdDeviation="3" result="coloredBlur" />
                  <feMerge>
                    <feMergeNode in="coloredBlur" />
                    <feMergeNode in="SourceGraphic" />
                  </feMerge>
                </filter>
              </defs>
              {connections.map((conn) => {
                const sourceModule = modules.find(m => m.id === conn.sourceModuleId);
                const targetModule = modules.find(m => m.id === conn.targetModuleId);

                if (!sourceModule || !targetModule) return null;

                const sourceY = sourceModule.position * 5 + 60;
                const targetY = targetModule.position * 5 + 60;
                const sourceX = 20;
                const targetX = sourceModule.hp * 25 + 10;

                return (
                  <path
                    key={conn.id}
                    d={`M ${sourceX} ${sourceY} C ${sourceX + 50} ${sourceY}, ${targetX - 50} ${targetY}, ${targetX} ${targetY}`}
                    className="cable"
                    stroke={MODULE_SPECS[sourceModule.type].color}
                    filter="url(#glow)"
                  />
                );
              })}
            </svg>

            {/* Modules */}
            {modules.map((module) => (
              <ModulePanel key={module.id} module={module} />
            ))}
          </div>

          {/* Right Rail */}
          <div className="absolute right-0 top-0 bottom-0 w-4 bg-gradient-to-l from-[#1a1d26] to-[#0f1014] border-l border-[#252530]">
            {Array.from({ length: Math.ceil(rackHeight / 80) }).map((_, i) => (
              <div
                key={i}
                className="rack-hole mx-auto my-4"
                style={{
                  boxShadow: i % 3 === 0 ? '0 0 4px #bf5fff22' : 'inset 1px 1px 2px rgba(0,0,0,0.8)',
                }}
              />
            ))}
          </div>
        </div>
      </div>

      {/* Status Bar */}
      <div className="h-6 bg-[#0f1014] border-t-2 border-[#252530] flex items-center px-4 text-xs text-[#484858] relative">
        {/* Decorative line */}
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-[#3a3a4a] to-transparent" />

        <span>CPU: <span className="text-[#00ff88]">0%</span></span>
        <span className="mx-2 text-[#252530]">|</span>
        <span>Audio: <span className="text-[#00d4ff]">48kHz / 32-bit</span></span>
        <span className="mx-2 text-[#252530]">|</span>
        <span className="text-[#00ff88] flex items-center gap-1">
          <div className="w-1.5 h-1.5 bg-[#00ff88] rounded-full animate-pulse" />
          Playing
        </span>
      </div>
    </div>
  );
}
