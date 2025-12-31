import React from 'react';

export default function ProcessScanner({ processes, selectedPid, onSelect, onScan, scanning }) {
  return (
    <div className="bg-white rounded-xl p-4 shadow-sm mx-6 mt-2 flex items-center gap-4 border border-gray-100">
      <div className="flex items-center gap-4 flex-1">
        <span className="text-gray-600 font-medium text-sm min-w-fit">目标进程</span>
        <div className="relative flex-1 group">
          <select 
            className="w-full bg-gray-50 border border-gray-200 rounded-lg px-4 py-2 text-sm text-gray-700 focus:outline-none focus:ring-2 focus:ring-primary/20 appearance-none transition-all cursor-pointer hover:border-gray-300"
            value={selectedPid || ''}
            onChange={(e) => onSelect(Number(e.target.value))}
            disabled={scanning}
          >
            <option value="" disabled>选择进程...</option>
            {processes.map((p) => (
              <option key={p.pid} value={p.pid}>
                {p.name} (PID: {p.pid})
              </option>
            ))}
          </select>
          <div className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-gray-400">
            ▼
          </div>
        </div>
      </div>
      
      <div className="flex items-center gap-2 min-w-[60px] justify-center">
         <span className="text-gray-400 text-xs font-medium">{scanning ? '扫描中...' : '等待中'}</span>
      </div>

      <button 
        onClick={onScan}
        disabled={scanning}
        className="px-6 py-2 bg-white border border-gray-200 text-gray-600 text-sm font-medium rounded-lg hover:bg-gray-50 hover:text-primary hover:border-primary/30 active:bg-gray-100 transition-all disabled:opacity-50 shadow-sm"
      >
        扫描
      </button>
    </div>
  );
}
