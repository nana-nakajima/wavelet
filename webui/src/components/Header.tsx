import React from 'react';
import { Icons } from './Icons';

export function Header() {
  return (
    <header className="h-14 bg-[#1e1e1e] border-b border-[#404040] flex items-center px-4 justify-between">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 bg-gradient-to-br from-[#4a9eff] to-[#a855f7] rounded pixel-button flex items-center justify-center">
            <span className="font-bold text-sm">W</span>
          </div>
          <h1 className="pixel-text font-bold text-lg">WAVEL</h1>
        </div>
        
        <div className="h-6 w-px bg-[#404040]" />
        
        <nav className="flex items-center gap-1">
          <button className="px-3 py-1 text-sm hover:bg-[#2d2d2d] rounded transition-colors">
            File
          </button>
          <button className="px-3 py-1 text-sm hover:bg-[#2d2d2d] rounded transition-colors">
            Edit
          </button>
          <button className="px-3 py-1 text-sm hover:bg-[#2d2d2d] rounded transition-colors">
            View
          </button>
          <button className="px-3 py-1 text-sm hover:bg-[#2d2d2d] rounded transition-colors">
            Transport
          </button>
        </nav>
      </div>
      
      <div className="flex items-center gap-2">
        {/* Transport controls */}
        <div className="flex items-center gap-1 mr-4">
          <button className="pixel-button p-2 rounded-sm">
            <Icons.Square />
          </button>
          <button className="pixel-button p-2 rounded-sm">
            <Icons.Play />
          </button>
        </div>
        
        {/* BPM Display */}
        <div className="bg-[#1a1a1a] px-3 py-1 rounded border border-[#404040] font-mono text-sm">
          <span className="text-[#4ade80]">120.00</span>
          <span className="text-[#737373] ml-1">BPM</span>
        </div>
        
        {/* Connection status */}
        <div className="flex items-center gap-2 text-xs text-[#737373]">
          <Icons.Wifi />
          <span>Connected</span>
        </div>
        
        <div className="h-6 w-px bg-[#404040]" />
        
        {/* Actions */}
        <button className="pixel-button p-2 rounded-sm" title="New Project">
          <Icons.FolderOpen />
        </button>
        <button className="pixel-button p-2 rounded-sm" title="Save">
          <Icons.Save />
        </button>
        <button className="pixel-button p-2 rounded-sm" title="Settings">
          <Icons.Settings />
        </button>
      </div>
    </header>
  );
}
