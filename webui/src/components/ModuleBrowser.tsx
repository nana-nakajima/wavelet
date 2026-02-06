import React from 'react';
import { useStore, MODULE_SPECS, ModuleType } from '../store';
import { Icons } from './Icons';

const MODULE_CATEGORIES = [
  { name: 'Sources', types: ['oscillator', 'lfo'] },
  { name: 'Filters', types: ['filter'] },
  { name: 'Envelopes', types: ['envelope', 'vca'] },
  { name: 'Effects', types: ['delay', 'reverb', 'compressor'] },
  { name: 'Outputs', types: ['output'] },
];

export function ModuleBrowser() {
  const { addModule } = useStore();
  const [search, setSearch] = React.useState('');
  const [expanded, setExpanded] = React.useState<Record<string, boolean>>({});

  const filteredCategories = MODULE_CATEGORIES.map(cat => ({
    ...cat,
    types: cat.types.filter(type => 
      MODULE_SPECS[type].name.toLowerCase().includes(search.toLowerCase())
    ),
  })).filter(cat => cat.types.length > 0 || search === '');

  return (
    <div className="w-64 bg-[#1e1e1e] border-r border-[#404040] flex flex-col">
      {/* Header */}
      <div className="p-3 border-b border-[#404040]">
        <h2 className="pixel-text font-bold text-sm mb-3">Modules</h2>
        <div className="relative">
          <Icons.Search />
          <input
            type="text"
            placeholder="Search modules..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-[#252525] border border-[#404040] rounded px-8 py-1 text-sm focus:outline-none focus:border-[#4a9eff]"
          />
        </div>
      </div>

      {/* Module List */}
      <div className="flex-1 overflow-y-auto">
        {filteredCategories.map((category) => (
          <div key={category.name} className="border-b border-[#2a2a2a]">
            <button
              onClick={() => setExpanded(prev => ({ ...prev, [category.name]: !prev[category.name] }))}
              className="w-full px-3 py-2 flex items-center justify-between hover:bg-[#2d2d2d] transition-colors"
            >
              <span className="text-sm font-medium text-[#a3a3a3]">{category.name}</span>
              <div className={`text-[#737373] transition-transform ${expanded[category.name] ? 'rotate-90' : ''}`}>
                <Icons.ChevronRight />
              </div>
            </button>
            
            {expanded[category.name] && (
              <div className="pb-2">
                {category.types.map((type) => {
                  const spec = MODULE_SPECS[type];
                  return (
                    <button
                      key={type}
                      onClick={() => addModule(type)}
                      className="w-full px-6 py-2 flex items-center gap-2 hover:bg-[#2d2d2d] transition-colors group"
                    >
                      <div 
                        className="w-3 h-3 rounded-sm" 
                        style={{ backgroundColor: spec.color }}
                      />
                      <span className="text-sm text-[#e5e5e5] group-hover:text-white">
                        {spec.name}
                      </span>
                      <span className="text-xs text-[#737373] ml-auto">
                        {spec.hp}HP
                      </span>
                      <div className="text-[#737373] opacity-0 group-hover:opacity-100 transition-opacity">
                        <Icons.Plus />
                      </div>
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Quick Add */}
      <div className="p-3 border-t border-[#404040]">
        <p className="text-xs text-[#737373] mb-2">Quick Add</p>
        <div className="flex flex-wrap gap-1">
          {(['oscillator', 'filter', 'envelope', 'output'] as ModuleType[]).map((type) => (
            <button
              key={type}
              onClick={() => addModule(type)}
              className="px-2 py-1 text-xs bg-[#252525] border border-[#404040] rounded hover:bg-[#2d2d2d] transition-colors"
            >
              {MODULE_SPECS[type].name}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
