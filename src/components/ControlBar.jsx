import React from 'react';
import clsx from 'clsx';

export default function ControlBar({ status, onApplyConfig, onStop, cpuInfo }) {
  const isActive = status === 'active';

  // 动态生成 CPU 信息显示
  const getCpuSummary = () => {
    if (!cpuInfo) return '加载中...';
    const cores = cpuInfo.cores || 0;
    const threads = cores; // 在这个项目中 cores 已经是线程数
    const physicalCores = Math.floor(cores / 2);
    return `${physicalCores}C/${threads}T [${cpuInfo.model || 'Unknown CPU'}]`;
  };

  return (
    <div className="mt-auto">
      <div className="px-6 py-6 flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <div className={clsx("w-2.5 h-2.5 rounded-full shadow-sm animate-pulse", isActive ? "bg-green-500 shadow-green-200" : "bg-red-500 shadow-red-200")}></div>
          <span className={clsx("font-bold text-sm uppercase tracking-wider", isActive ? "text-green-600" : "text-gray-400")}>
            {isActive ? 'ACTIVE' : 'STANDBY'}
          </span>
        </div>

        <div className="flex gap-3">
          <button
            onClick={onApplyConfig}
            className="px-8 py-2.5 bg-primary hover:bg-primary-hover text-white rounded-lg font-medium shadow-md shadow-blue-200 transition-all active:scale-95 hover:-translate-y-0.5"
          >
            应用配置
          </button>
          <button
            onClick={onStop}
            disabled={!isActive}
            className="px-8 py-2.5 bg-white border border-gray-200 text-gray-500 rounded-lg font-medium hover:bg-gray-50 hover:text-red-500 disabled:opacity-50 disabled:hover:bg-white disabled:hover:text-gray-500 transition-all active:scale-95"
          >
            停止
          </button>
        </div>
      </div>

      {/* Bottom Status Bar */}
      <div className="bg-white border-t border-gray-200 px-6 py-2.5 flex justify-between items-center text-[10px] text-gray-400 font-medium">
        <span className="tracking-wide">{getCpuSummary()}</span>
        <div className="flex items-center gap-2">
          <div className={clsx("w-1.5 h-1.5 rounded-full", isActive ? "bg-green-500" : "bg-gray-300")}></div>
          <span>{isActive ? '已激活' : '未激活'}</span>
        </div>
      </div>
    </div>
  );
}

