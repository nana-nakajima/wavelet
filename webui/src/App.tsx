import React from 'react';
import { Rack } from './components/Rack';
import { ModuleBrowser } from './components/ModuleBrowser';
import { Header } from './components/Header';
import { useStore } from './store';

export function App() {
  const { modules, addModule, removeModule } = useStore();

  return (
    <div className="h-screen flex flex-col bg-[#1a1a1a] text-[#e5e5e5] overflow-hidden">
      {/* Header */}
      <Header />
      
      <div className="flex-1 flex overflow-hidden">
        {/* Left: Module Browser */}
        <ModuleBrowser />
        
        {/* Center: Rack */}
        <Rack />
        
        {/* Right: Parameters Panel */}
        <div className="w-80 bg-[#252525] border-l border-[#404040] p-4">
          <h2 className="pixel-text text-lg font-bold mb-4">Parameters</h2>
          <p className="text-[#737373] text-sm">
            Select a module to edit its parameters
          </p>
        </div>
      </div>
    </div>
  );
}
