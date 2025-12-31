import React from 'react';
import clsx from 'clsx';

export default function CoreGrid({ cores, selectedCores, onToggleCore, onSelectAll, onSelectNone, onSelectPhysical, onSelectSMT, onApply }) {
  
  return (
    <div className="bg-white rounded-xl p-6 shadow-sm mx-6 mt-4 border border-gray-100">
      <div className="flex items-center justify-between mb-6">
        <span className="text-gray-600 font-medium text-sm">允许调度池</span>
        <div className="flex gap-2">
          <button onClick={onSelectPhysical} className="px-3 py-1.5 text-xs font-medium bg-gray-50 text-gray-600 border border-gray-200 rounded hover:bg-gray-100 hover:text-primary transition-colors">物理核心</button>
          <button onClick={onSelectSMT} className="px-3 py-1.5 text-xs font-medium bg-gray-50 text-gray-600 border border-gray-200 rounded hover:bg-gray-100 hover:text-primary transition-colors">SMT</button>
          <button onClick={onSelectAll} className="px-3 py-1.5 text-xs font-medium bg-gray-50 text-gray-600 border border-gray-200 rounded hover:bg-gray-100 hover:text-primary transition-colors">全选</button>
          <button onClick={onSelectNone} className="px-3 py-1.5 text-xs font-medium bg-gray-50 text-gray-600 border border-gray-200 rounded hover:bg-gray-100 hover:text-primary transition-colors">清空</button>
          <button onClick={onApply} className="px-5 py-1.5 text-xs font-medium bg-primary text-white rounded hover:bg-primary-hover shadow-md shadow-blue-200 transition-all active:scale-95 ml-2">应用</button>
        </div>
      </div>

      <div className="grid grid-cols-8 gap-3">
        {cores.map((core, index) => {
          const isSelected = selectedCores.includes(index);
          const isPhysical = index % 2 === 0;
          
          return (
            <button
              key={index}
              onClick={() => onToggleCore(index)}
              className={clsx(
                "flex flex-col items-center justify-center py-3 rounded-lg transition-all duration-200 border relative overflow-hidden group",
                isSelected 
                  ? "bg-primary border-primary text-white shadow-md shadow-blue-200 scale-105 z-10" 
                  : "bg-gray-50 border-gray-100 text-gray-500 hover:bg-white hover:border-gray-300 hover:shadow-sm"
              )}
            >
              <span className="text-sm font-bold z-10 relative">核心 {index}</span>
              <span className={clsx("text-[10px] mt-0.5 z-10 relative font-medium", isSelected ? "text-blue-100" : "text-gray-400 group-hover:text-gray-500")}>
                {isPhysical ? '物理核心' : 'SMT'}
              </span>
              {/* Glossy effect for selected items */}
              {isSelected && <div className="absolute top-0 left-0 w-full h-1/2 bg-white/10 pointer-events-none"></div>}
            </button>
          );
        })}
      </div>
    </div>
  );
}
