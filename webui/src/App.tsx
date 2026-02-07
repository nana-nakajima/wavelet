import React from 'react';
import { Rack } from './components/Rack';
import { ModuleBrowser } from './components/ModuleBrowser';
import { Header } from './components/Header';
import { useStore } from './store';

export function App() {
  const { modules, selectedModuleId } = useStore();
  const selectedModule = modules.find(m => m.id === selectedModuleId);

  return (
    <div className="h-screen flex flex-col bg-[#0a0a0c] text-[#b8b8b8] overflow-hidden">
      {/* Header */}
      <Header />

      <div className="flex-1 flex overflow-hidden">
        {/* Left: Module Browser */}
        <ModuleBrowser />

        {/* Center: Rack */}
        <Rack />

        {/* Right: Parameters Panel */}
        <div className="w-80 bg-[#0f1014] border-l-2 border-[#252530] p-4 relative">
          {/* Decorative corner */}
          <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-[#bf5fff] via-transparent to-transparent opacity-30" />

          <h2 className="pixel-text text-sm font-bold mb-4 text-[#bf5fff]">
            Parameters
          </h2>

          {selectedModule ? (
            <div className="space-y-4">
              <div className="text-xs text-[#686878]">
                Selected: <span className="text-[#00ff88]">{selectedModule.type.toUpperCase()}</span>
              </div>
              <div className="text-xs text-[#484858]">
                Module ID: {selectedModule.id}
              </div>
            </div>
          ) : (
            <p className="text-[#484858] text-sm">
              Select a module to edit its parameters
            </p>
          )}

          {/* Decorative bottom element */}
          <div className="absolute bottom-4 left-4 right-4 h-px bg-gradient-to-r from-transparent via-[#3a3a4a] to-transparent" />
        </div>
      </div>
    </div>
  );
}
