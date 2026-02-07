import React from 'react';
import { Icons } from './Icons';

export function Header() {
  return (
    <header className="h-14 bg-[#0f1014] border-b-2 border-[#252530] flex items-center px-4 justify-between relative">
      {/* Top glow line */}
      <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-[#00ff88] to-transparent opacity-30" />

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 bg-gradient-to-br from-[#00ff88] to-[#bf5fff] rounded-sm pixel-button flex items-center justify-center relative overflow-hidden">
            <span className="font-bold text-sm text-[#0a0a0c] relative z-10">W</span>
            {/* Inner glow */}
            <div className="absolute inset-0 bg-gradient-to-t from-transparent to-white opacity-20" />
          </div>
          <h1 className="pixel-text font-bold text-lg text-[#00ff88]">WAVELET</h1>
        </div>

        <div className="h-6 w-px bg-[#252530]" />

        <nav className="flex items-center gap-1">
          {['File', 'Edit', 'View', 'Transport'].map((item) => (
            <button
              key={item}
              className="px-3 py-1 text-sm text-[#686878] hover:text-[#00ff88] hover:bg-[#1a1d26] rounded transition-all duration-150"
            >
              {item}
            </button>
          ))}
        </nav>
      </div>

      <div className="flex items-center gap-2">
        {/* Transport controls */}
        <div className="flex items-center gap-1 mr-4">
          <button className="pixel-button p-2 rounded-sm hover:text-[#ff3366]">
            <Icons.Square />
          </button>
          <button className="pixel-button p-2 rounded-sm hover:text-[#00ff88]">
            <Icons.Play />
          </button>
        </div>

        {/* BPM Display */}
        <div className="bg-[#0a0a0c] px-3 py-1 rounded-sm border-2 border-[#252530] font-mono text-sm relative overflow-hidden">
          <span className="text-[#00ff88] relative z-10">120.00</span>
          <span className="text-[#484858] ml-1 relative z-10">BPM</span>
          {/* Scanline effect */}
          <div className="absolute inset-0 bg-gradient-to-b from-transparent via-[#00ff8808] to-transparent animate-pulse" />
        </div>

        {/* Connection status */}
        <div className="flex items-center gap-2 text-xs text-[#484858]">
          <div className="w-2 h-2 bg-[#00ff88] rounded-full animate-pulse shadow-[0_0_8px_#00ff88]" />
          <span>Connected</span>
        </div>

        <div className="h-6 w-px bg-[#252530]" />

        {/* Actions */}
        <button className="pixel-button p-2 rounded-sm hover:text-[#00d4ff]" title="New Project">
          <Icons.FolderOpen />
        </button>
        <button className="pixel-button p-2 rounded-sm hover:text-[#ffcc00]" title="Save">
          <Icons.Save />
        </button>
        <button className="pixel-button p-2 rounded-sm hover:text-[#bf5fff]" title="Settings">
          <Icons.Settings />
        </button>
      </div>
    </header>
  );
}
