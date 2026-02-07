import React from 'react';
import { Module, MODULE_SPECS } from '../store';
import { Knob } from './Knob';
import { Port } from './Port';
import { Icons } from './Icons';
import { useStore } from '../store';

interface ModulePanelProps {
  module: Module;
}

export function ModulePanel({ module }: ModulePanelProps) {
  const { selectModule, selectedModuleId, removeModule, updateModuleParam } = useStore();
  const spec = MODULE_SPECS[module.type];
  const isSelected = selectedModuleId === module.id;
  const [expanded, setExpanded] = React.useState(true);

  // Calculate position (HP = horizontal pitch units, each unit = 25px)
  const width = module.hp * 25;
  const topPosition = module.position * 5;

  return (
    <div
      className={`absolute module-panel rounded-sm transition-all ${
        isSelected ? 'ring-2 ring-[#00ff88] selected' : ''
      }`}
      style={{
        left: 10,
        top: topPosition,
        width: width,
        minHeight: expanded ? 'auto' : '60px',
        zIndex: isSelected ? 10 : 1,
        '--module-color': spec.color,
      } as React.CSSProperties}
      onClick={(e) => {
        e.stopPropagation();
        selectModule(module.id);
      }}
    >
      {/* Top glow bar */}
      <div
        className="absolute top-0 left-0 right-0 h-[2px] opacity-60"
        style={{
          background: `linear-gradient(90deg, transparent 0%, ${spec.color} 20%, ${spec.color} 80%, transparent 100%)`,
          boxShadow: `0 0 10px ${spec.color}`,
        }}
      />

      {/* Header */}
      <div
        className="h-8 px-2 flex items-center justify-between border-b border-[#252530] cursor-pointer"
        onClick={(e) => {
          e.stopPropagation();
          setExpanded(!expanded);
        }}
      >
        <div className="flex items-center gap-2">
          <div
            className="w-2 h-2 rounded-sm"
            style={{
              backgroundColor: spec.color,
              boxShadow: `0 0 6px ${spec.color}`,
            }}
          />
          <span className="text-xs font-bold pixel-text" style={{ color: spec.color }}>
            {spec.name}
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={(e) => {
              e.stopPropagation();
              removeModule(module.id);
            }}
            className="p-1 hover:bg-[#1a1d26] rounded transition-colors"
          >
            <div className="text-[#484858] hover:text-[#ff3366] transition-colors">
              <Icons.Trash2 />
            </div>
          </button>
          <div className="text-[#484858]">
            {expanded ? <Icons.ChevronUp /> : <Icons.ChevronDown />}
          </div>
        </div>
      </div>

      {expanded && (
        <div className="p-3 space-y-3">
          {/* Inputs Row */}
          {spec.inputs.length > 0 && (
            <div className="flex items-center gap-2">
              <span className="text-[10px] text-[#484858] w-8 uppercase tracking-wider">In</span>
              <div className="flex-1 flex gap-1 justify-end">
                {spec.inputs.map((input, i) => (
                  <Port
                    key={input}
                    id={`input-${i}`}
                    label={input}
                    type="input"
                    color={spec.color}
                    moduleId={module.id}
                  />
                ))}
              </div>
            </div>
          )}

          {/* Parameters */}
          <div className="flex flex-wrap gap-2 justify-center">
            {spec.params.slice(0, 3).map((param) => (
              <Knob
                key={param}
                label={param.charAt(0).toUpperCase() + param.slice(1)}
                value={module.params[param] || 0.5}
                color={spec.color}
                size={36}
                onChange={(value) => updateModuleParam(module.id, param, value)}
              />
            ))}
          </div>

          {/* Extra params */}
          {spec.params.length > 3 && (
            <div className="flex flex-wrap gap-2 justify-center">
              {spec.params.slice(3).map((param) => (
                <Knob
                  key={param}
                  label={param.charAt(0).toUpperCase() + param.slice(1)}
                  value={module.params[param] || 0.5}
                  color={spec.color}
                  size={28}
                  onChange={(value) => updateModuleParam(module.id, param, value)}
                />
              ))}
            </div>
          )}

          {/* Outputs Row */}
          {spec.outputs.length > 0 && (
            <div className="flex items-center gap-2">
              <span className="text-[10px] text-[#484858] w-8 uppercase tracking-wider">Out</span>
              <div className="flex-1 flex gap-1">
                {spec.outputs.map((output, i) => (
                  <Port
                    key={output}
                    id={`output-${i}`}
                    label={output}
                    type="output"
                    color={spec.color}
                    moduleId={module.id}
                  />
                ))}
              </div>
            </div>
          )}

          {/* Power LED */}
          <div className="flex justify-center pt-2 border-t border-[#1a1d26]">
            <div className="flex items-center gap-2">
              <span className="text-[10px] text-[#484858] uppercase tracking-wider">Pwr</span>
              <div
                className="led on w-2 h-2"
                style={{
                  backgroundColor: spec.color,
                  boxShadow: `0 0 8px ${spec.color}`,
                }}
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
