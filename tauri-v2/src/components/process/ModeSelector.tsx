import React from 'react';
import { Zap, Scale, Settings } from 'lucide-react';

interface ModeSelectorProps {
    mode: string;
    onModeClick: (id: string) => void;
}

export const ModeSelector: React.FC<ModeSelectorProps> = ({ mode, onModeClick }) => {
    const modes = [
        { id: 'dynamic', label: 'T mode1', icon: Zap },
        { id: 'd2', label: 'T mode2', icon: Scale, note: '笔记本可用' },
        { id: 'd3', label: 'T mode3', icon: Zap },
        { id: 'custom', label: '自定义', icon: Settings, note: '高级配置' },
    ];

    return (
        <div className="glass rounded-xl p-4 shadow-sm border border-violet-100/50 bg-white/40 backdrop-blur-sm">
            <div className="grid grid-cols-4 gap-3">
                {modes.map((m) => {
                    const isActive = mode === m.id;
                    const Icon = m.icon;

                    return (
                        <button
                            key={m.id}
                            onClick={() => onModeClick(m.id)}
                            className={`relative p-3 rounded-lg text-center transition-all duration-200 flex flex-col items-center justify-center gap-1.5 ${isActive
                                ? 'bg-gradient-to-br from-violet-500 to-pink-500 text-white shadow-glow'
                                : 'bg-white hover:bg-slate-50 text-slate-600 border border-slate-200/60'
                                }`}
                        >
                            <Icon size={16} className={isActive ? 'text-white' : 'text-violet-500'} />
                            <span className="font-medium text-xs">{m.label}</span>
                            {m.note && isActive && (
                                <span className="text-[10px] opacity-80 scale-90">({m.note})</span>
                            )}
                        </button>
                    );
                })}
            </div>
        </div>
    );
};
