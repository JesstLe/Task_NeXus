import React, { useEffect, useRef } from 'react';
import { Zap, Gauge, Cpu, Check, XCircle } from 'lucide-react';

export default function ContextMenu({ x, y, process, onClose, onAction }) {
    const menuRef = useRef(null);

    useEffect(() => {
        const handleClickOutside = (event) => {
            if (menuRef.current && !menuRef.current.contains(event.target)) {
                onClose();
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, [onClose]);

    if (!process) return null;

    return (
        <div
            ref={menuRef}
            className="fixed z-50 bg-white/95 backdrop-blur-xl rounded-xl shadow-2xl border border-slate-200/50 py-1.5 min-w-[200px] animate-in fade-in zoom-in-95 duration-150 origin-top-left"
            style={{ top: y, left: x }}
        >
            <div className="px-3 py-1.5 border-b border-slate-100 mb-1">
                <div className="font-semibold text-sm text-slate-800 truncate max-w-[180px]">{process.name}</div>
                <div className="text-[10px] text-slate-400 font-mono">PID: {process.pid}</div>
            </div>

            <div className="px-1 space-y-0.5">
                <button
                    onClick={() => onAction('addToHigh')}
                    className="w-full text-left px-2 py-1.5 rounded-lg text-xs font-medium text-slate-700 hover:bg-violet-50 hover:text-violet-600 flex items-center gap-2 transition-colors"
                >
                    <Zap size={14} className="text-violet-500" />
                    <span>总是高优先级 (游戏模式)</span>
                </button>

                <button
                    onClick={() => onAction('addToLow')}
                    className="w-full text-left px-2 py-1.5 rounded-lg text-xs font-medium text-slate-700 hover:bg-red-50 hover:text-red-600 flex items-center gap-2 transition-colors"
                >
                    <Gauge size={14} className="text-red-500" />
                    <span>总是低优先级 (压制模式)</span>
                </button>

                <div className="h-px bg-slate-100 my-1 mx-2" />

                <button
                    onClick={() => onAction('bindE')}
                    className="w-full text-left px-2 py-1.5 rounded-lg text-xs font-medium text-slate-700 hover:bg-green-50 hover:text-green-600 flex items-center gap-2 transition-colors"
                >
                    <Cpu size={14} className="text-green-500" />
                    <span>锁定到小核 (Efficiency)</span>
                </button>

                <button
                    onClick={() => onAction('bindP')}
                    className="w-full text-left px-2 py-1.5 rounded-lg text-xs font-medium text-slate-700 hover:bg-blue-50 hover:text-blue-600 flex items-center gap-2 transition-colors"
                >
                    <Cpu size={14} className="text-blue-500" />
                    <span>锁定到大核 (Performance)</span>
                </button>

                <div className="h-px bg-slate-100 my-1 mx-2" />

                <button
                    onClick={() => onAction('reset')}
                    className="w-full text-left px-2 py-1.5 rounded-lg text-xs font-medium text-slate-500 hover:bg-slate-100 flex items-center gap-2 transition-colors"
                >
                    <XCircle size={14} />
                    <span>重置所有规则</span>
                </button>
            </div>
        </div>
    );
}
