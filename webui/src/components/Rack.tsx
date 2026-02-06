import React from 'react';
import { useStore, MODULE_SPECS } from '../store';
import { ModulePanel } from './ModulePanel';
import { Icons } from './Icons';

export function Rack() {
  const { modules, connections, removeModule } = useStore();

  // Calculate rack height based on modules
  const rackHeight = Math.max(800, modules.length * 120 + 200);

  return (
    <div className="flex-1 flex flex-col overflow-hidden bg-[#1a1a1a]">
      {/* Rack Header */}
      <div className="h-10 bg-[#252525] border-b border-[#404040] flex items-center px-4">
        <div className="flex-1 flex items-center gap-4">
          <span className="text-xs text-[#737373]">RACK</span>
          <span className="text-xs text-[#737373]">
            {modules.length} module{modules.length !== 1 ? 's' : ''}
          </span>
          {modules.length === 0 && (
            <span className="text-xs text-[#737373] italic">
              Add modules from the browser
            </span>
          )}
        </div>
        
        <div className="flex items-center gap-2">
          <button 
            onClick={() => modules.forEach(m => removeModule(m.id))}
            className="pixel-button px-2 py-1 text-xs flex items-center gap-1 text-[#ef4444] hover:text-[#f87171]"
            disabled={modules.length === 0}
          >
            <Icons.Trash2 />
            Clear All
          </button>
        </div>
      </div>

      {/* Rack Mount */}
      <div className="flex-1 overflow-auto relative">
        {/* Rack Rails */}
        <div 
          className="min-h-full bg-[#151515] relative"
          style={{ 
            backgroundImage: `
              repeating-linear-gradient(
                0deg,
                transparent,
                transparent 79px,
                #2a2a2a 79px,
                #2a2a2a 80px
              )
            `,
          }}
        >
          {/* Left Rail */}
          <div className="absolute left-0 top-0 bottom-0 w-4 bg-gradient-to-r from-[#2a2a2a] to-[#1a1a1a]">
            {/* Mount holes */}
            {Array.from({ length: Math.ceil(rackHeight / 80) }).map((_, i) => (
              <div key={i} className="rack-hole mx-auto my-4" />
            ))}
          </div>

          {/* Modules Container */}
          <div className="absolute left-4 right-4 top-0">
            {/* Rails overlay */}
            <div className="absolute inset-y-0 left-0 right-0 border-l-2 border-r-2 border-[#404040] pointer-events-none" />
            
            {/* Cable Layer */}
            <svg className="absolute inset-0 w-full h-full pointer-events-none z-0">
              {connections.map((conn) => {
                // Find module positions
                const sourceModule = modules.find(m => m.id === conn.sourceModuleId);
                const targetModule = modules.find(m => m.id === conn.targetModuleId);
                
                if (!sourceModule || !targetModule) return null;
                
                const sourceY = sourceModule.position * 5 + 60;
                const targetY = targetModule.position * 5 + 60;
                
                // Calculate X positions
                const sourceX = 20; // Left side of module
                const targetX = sourceModule.hp * 25 + 10; // Right side
                
                return (
                  <path
                    key={conn.id}
                    d={`M ${sourceX} ${sourceY} C ${sourceX + 50} ${sourceY}, ${targetX - 50} ${targetY}, ${targetX} ${targetY}`}
                    className="cable"
                    stroke={MODULE_SPECS[sourceModule.type].color}
                    filter={`drop-shadow(0 0 4px ${MODULE_SPECS[sourceModule.type].color})`}
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
          <div className="absolute right-0 top-0 bottom-0 w-4 bg-gradient-to-l from-[#2a2a2a] to-[#1a1a1a]">
            {Array.from({ length: Math.ceil(rackHeight / 80) }).map((_, i) => (
              <div key={i} className="rack-hole mx-auto my-4" />
            ))}
          </div>
        </div>
      </div>

      {/* Status Bar */}
      <div className="h-6 bg-[#1e1e1e] border-t border-[#404040] flex items-center px-4 text-xs text-[#737373]">
        <span>CPU: 0%</span>
        <span className="mx-2">|</span>
        <span>Audio: 48kHz / 32-bit</span>
        <span className="mx-2">|</span>
        <span className="text-[#4ade80]">Playing</span>
      </div>
    </div>
  );
}
