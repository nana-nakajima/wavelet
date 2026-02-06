import { create } from 'zustand';

// Simple ID generator
const generateId = () => Math.random().toString(36).substring(2, 11);

export type ModuleType = 
  | 'oscillator'
  | 'filter'
  | 'envelope'
  | 'lfo'
  | 'delay'
  | 'reverb'
  | 'compressor'
  | 'output'
  | 'vca';

export interface Module {
  id: string;
  type: ModuleType;
  position: number; // Vertical position in rack (0-14 for 14HP)
  params: Record<string, number>;
  connections: Connection[];
}

export interface Connection {
  id: string;
  sourceModuleId: string;
  sourcePort: string;
  targetModuleId: string;
  targetPort: string;
}

interface AppState {
  modules: Module[];
  connections: Connection[];
  selectedModuleId: string | null;
  hoveredPort: { moduleId: string; port: string; type: 'input' | 'output' } | null;
  
  addModule: (type: ModuleType) => void;
  removeModule: (id: string) => void;
  selectModule: (id: string | null) => void;
  updateModuleParam: (id: string, param: string, value: number) => void;
  addConnection: (connection: Omit<Connection, 'id'>) => void;
  removeConnection: (id: string) => void;
  setHoveredPort: (port: { moduleId: string; port: string; type: 'input' | 'output' } | null) => void;
}

const MODULE_SPECS: Record<ModuleType, { name: string; hp: number; color: string; inputs: string[]; outputs: string[]; params: string[] }> = {
  oscillator: {
    name: 'VCO',
    hp: 10,
    color: '#4a9eff',
    inputs: ['FM', 'Sync'],
    outputs: ['Sine', 'Saw', 'Square'],
    params: ['frequency', 'detune', 'mix'],
  },
  filter: {
    name: 'VCF',
    hp: 8,
    color: '#4ade80',
    inputs: ['Audio', 'Cutoff CV'],
    outputs: ['LP', 'HP', 'BP'],
    params: ['cutoff', 'resonance', 'drive'],
  },
  envelope: {
    name: 'ADSR',
    hp: 6,
    color: '#f97316',
    inputs: ['Gate'],
    outputs: ['Envelope'],
    params: ['attack', 'decay', 'sustain', 'release'],
  },
  lfo: {
    name: 'LFO',
    hp: 6,
    color: '#a855f7',
    inputs: [],
    outputs: ['LFO'],
    params: ['rate', 'waveform', 'amplitude'],
  },
  delay: {
    name: 'Delay',
    hp: 8,
    color: '#4ade80',
    inputs: ['Audio', 'Time', 'Feedback'],
    outputs: ['Wet', 'Dry'],
    params: ['time', 'feedback', 'mix'],
  },
  reverb: {
    name: 'Reverb',
    hp: 10,
    color: '#f97316',
    inputs: ['Audio'],
    outputs: ['Wet', 'Dry'],
    params: ['size', 'decay', 'mix', 'damping'],
  },
  compressor: {
    name: 'Compressor',
    hp: 6,
    color: '#ef4444',
    inputs: ['Audio'],
    outputs: ['Audio'],
    params: ['threshold', 'ratio', 'attack', 'release'],
  },
  output: {
    name: 'Output',
    hp: 6,
    color: '#eab308',
    inputs: ['Left', 'Right', 'Aux'],
    outputs: [],
    params: ['volume', 'pan'],
  },
  vca: {
    name: 'VCA',
    hp: 4,
    color: '#a855f7',
    inputs: ['Audio', 'CV'],
    outputs: ['Audio'],
    params: ['gain'],
  },
};

export const useStore = create<AppState>((set, get) => ({
  modules: [],
  connections: [],
  selectedModuleId: null,
  hoveredPort: null,

  addModule: (type) => {
    const spec = MODULE_SPECS[type];
    const id = generateId();
    
    // Find next available position
    const positions = get().modules.map(m => m.position + m.hp);
    let position = 0;
    while (positions.includes(position) || positions.includes(position - 1)) {
      position += 2;
    }
    
    const module: Module = {
      id,
      type,
      position,
      params: Object.fromEntries(spec.params.map(p => [p, 0.5])),
      connections: [],
    };
    
    set((state) => ({ modules: [...state.modules, module] }));
  },

  removeModule: (id) => {
    set((state) => ({
      modules: state.modules.filter((m) => m.id !== id),
      connections: state.connections.filter(
        (c) => c.sourceModuleId !== id && c.targetModuleId !== id
      ),
      selectedModuleId: state.selectedModuleId === id ? null : state.selectedModuleId,
    }));
  },

  selectModule: (id) => set({ selectedModuleId: id }),

  updateModuleParam: (id, param, value) => {
    set((state) => ({
      modules: state.modules.map((m) =>
        m.id === id ? { ...m, params: { ...m.params, [param]: value } } : m
      ),
    }));
  },

  addConnection: (connection) => {
    const id = generateId();
    set((state) => ({
      connections: [...state.connections, { ...connection, id }],
    }));
  },

  removeConnection: (id) => {
    set((state) => ({
      connections: state.connections.filter((c) => c.id !== id),
    }));
  },

  setHoveredPort: (port) => set({ hoveredPort: port }),
}));

export { MODULE_SPECS };
