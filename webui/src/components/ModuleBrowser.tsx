import React from 'react';
import { useStore, MODULE_SPECS, ModuleType } from '../store';
import { Icons } from './Icons';

const MODULE_CATEGORIES = [
  { name: 'Sources', types: ['oscillator', 'lfo'], color: '#00d4ff' },
  { name: 'Filters', types: ['filter'], color: '#00ff88' },
  { name: 'Envelopes', types: ['envelope', 'vca'], color: '#ff6b35' },
  { name: 'Effects', types: ['delay', 'reverb', 'compressor'], color: '#bf5fff' },
  { name: 'Outputs', types: ['output'], color: '#ffcc00' },
];

export function ModuleBrowser() {
  const { addModule } = useStore();
  const [search, setSearch] = React.useState('');
  const [expanded, setExpanded] = React.useState<Record<string, boolean>>({
    Sources: true,
    Filters: true,
  });

  const filteredCategories = MODULE_CATEGORIES.map(cat => ({
    ...cat,
    types: cat.types.filter(type =>
      MODULE_SPECS[type as ModuleType].name.toLowerCase().includes(search.toLowerCase())
    ),
  })).filter(cat => cat.types.length > 0 || search === '');

  return (
    <div className="w-64 bg-[#0f1014] border-r-2 border-[#252530] flex flex-col relative">
      {/* Decorative side glow */}
      <div className="absolute top-0 right-0 w-px h-full bg-gradient-to-b from-[#00ff88] via-transparent to-[#bf5fff] opacity-20" />

      {/* Header */}
      <div className="p-3 border-b-2 border-[#252530]">
        <h2 className="pixel-text font-bold text-sm mb-3 text-[#00ff88]">Modules</h2>
        <div className="relative">
          <div className="absolute left-2 top-1/2 -translate-y-1/2 text-[#484858]">
            <Icons.Search />
          </div>
          <input
            type="text"
            placeholder="Search modules..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-[#0a0a0c] border-2 border-[#252530] rounded-sm px-8 py-1.5 text-sm text-[#b8b8b8] placeholder-[#484858] focus:outline-none focus:border-[#00ff88] transition-colors"
          />
        </div>
      </div>

      {/* Module List */}
      <div className="flex-1 overflow-y-auto">
        {filteredCategories.map((category) => (
          <div key={category.name} className="border-b border-[#1a1d26]">
            <button
              onClick={() => setExpanded(prev => ({ ...prev, [category.name]: !prev[category.name] }))}
              className="w-full px-3 py-2 flex items-center justify-between hover:bg-[#1a1d26] transition-colors group"
            >
              <div className="flex items-center gap-2">
                <div
                  className="w-2 h-2 rounded-sm"
                  style={{ backgroundColor: category.color, boxShadow: `0 0 6px ${category.color}55` }}
                />
                <span className="text-sm font-medium text-[#686878] group-hover:text-[#b8b8b8]">
                  {category.name}
                </span>
              </div>
              <div className={`text-[#484858] transition-transform duration-200 ${expanded[category.name] ? 'rotate-90' : ''}`}>
                <Icons.ChevronRight />
              </div>
            </button>

            {expanded[category.name] && (
              <div className="pb-2">
                {category.types.map((type) => {
                  const spec = MODULE_SPECS[type as ModuleType];
                  return (
                    <button
                      key={type}
                      onClick={() => addModule(type as ModuleType)}
                      className="w-full px-6 py-2 flex items-center gap-2 hover:bg-[#1a1d26] transition-all group"
                    >
                      <div
                        className="w-3 h-3 rounded-sm transition-all group-hover:scale-110"
                        style={{
                          backgroundColor: spec.color,
                          boxShadow: `0 0 8px ${spec.color}44`,
                        }}
                      />
                      <span className="text-sm text-[#b8b8b8] group-hover:text-white transition-colors">
                        {spec.name}
                      </span>
                      <span className="text-xs text-[#484858] ml-auto">
                        {spec.hp}HP
                      </span>
                      <div className="text-[#484858] opacity-0 group-hover:opacity-100 group-hover:text-[#00ff88] transition-all">
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
      <div className="p-3 border-t-2 border-[#252530] bg-[#0a0a0c]">
        <p className="text-xs text-[#484858] mb-2 uppercase tracking-wider">Quick Add</p>
        <div className="flex flex-wrap gap-1">
          {(['oscillator', 'filter', 'envelope', 'output'] as ModuleType[]).map((type) => (
            <button
              key={type}
              onClick={() => addModule(type)}
              className="px-2 py-1 text-xs bg-[#1a1d26] border border-[#252530] rounded-sm hover:border-[#00ff88] hover:text-[#00ff88] transition-all"
              style={{
                boxShadow: `0 0 0 0 ${MODULE_SPECS[type].color}`,
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.boxShadow = `0 0 8px ${MODULE_SPECS[type].color}44`;
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.boxShadow = 'none';
              }}
            >
              {MODULE_SPECS[type].name}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
