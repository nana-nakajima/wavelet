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
        isSelected ? 'ring-2 ring-[#4a9eff]' : ''
      }`}
      style={{
        left: 10,
        top: topPosition,
        width: width,
        minHeight: expanded ? 'auto' : '60px',
        zIndex: isSelected ? 10 : 1,
      }}
      onClick={(e) => {
        e.stopPropagation();
        selectModule(module.id);
      }}
    >
      {/* Header */}
      <div className="h-8 px-2 flex items-center justify-between border-b border-[#404040] cursor-pointer"
        onClick={(e) => {
          e.stopPropagation();
          setExpanded(!expanded);
        }}
      >
        <div className="flex items-center gap-2">
          <div 
            className="w-2 h-2 rounded-full" 
            style={{ backgroundColor: spec.color }}
          />
          <span className="text-xs font-bold pixel-text">{spec.name}</span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={(e) => {
              e.stopPropagation();
              removeModule(module.id);
            }}
            className="p-1 hover:bg-[#404040] rounded transition-colors"
          >
            <div className="text-[#737373] hover:text-[#ef4444]">
              <Icons.Trash2 />
            </div>
          </button>
          {expanded ? <Icons.ChevronUp /> : <Icons.ChevronDown />}
        </div>
      </div>

      {expanded && (
        <div className="p-3 space-y-3">
          {/* Inputs Row */}
          {spec.inputs.length > 0 && (
            <div className="flex items-center gap-2">
              <span className="text-[10px] text-[#737373] w-8">IN</span>
              <div className="flex-1 flex gap-1 justify-end">
                {spec.inputs.map((input, i) => (
                  <Port
                    key={input}
                    id={`input-${i}`}
                    label={input}
                    type="input"
                    color={spec.color}
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
              <span className="text-[10px] text-[#737373] w-8">OUT</span>
              <div className="flex-1 flex gap-1">
                {spec.outputs.map((output, i) => (
                  <Port
                    key={output}
                    id={`output-${i}`}
                    label={output}
                    type="output"
                    color={spec.color}
                  />
                ))}
              </div>
            </div>
          )}

          {/* Power LED */}
          <div className="flex justify-center pt-2 border-t border-[#2a2a2a]">
            <div className="flex items-center gap-2">
              <span className="text-[10px] text-[#737373]">PWR</span>
              <div className="led on w-2 h-2 rounded-full" />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
