import React, { useState } from 'react';
import { Trash2, Check, AlertTriangle, Search, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface RegistryItem {
    id: string;
    name: string;
    category: string;
    count: number;
}

// Mock Data - In real implementation, this would come from Backend
const REGISTRY_CATEGORIES: RegistryItem[] = [
    { id: 'invalid_plugins', name: '无效的插件以及组件', category: '系统', count: 0 },
    { id: 'deleted_file_paths', name: '已删除文件残留的无效路径', category: '系统', count: 0 },
    { id: 'invalid_program_refs', name: '已注册程序残留的无效路径', category: '系统', count: 0 },
    { id: 'file_extension_info', name: '无效的文件扩展名信息', category: '系统', count: 0 },
    { id: 'help_docs', name: '无效的帮助文档', category: '系统', count: 0 },
    { id: 'firewall_settings', name: '无效的Windows防火墙设置', category: '安全', count: 0 },
    { id: 'font_info', name: '无效的字体信息', category: '系统', count: 0 },
    { id: 'dynamic_links', name: '丢失的动态链接库', category: '系统', count: 0 },
    { id: 'history_records', name: '用户操作历史记录', category: '隐私', count: 0 },
    { id: 'invalid_uninstall', name: '无效的卸载信息', category: '软件', count: 0 },
    { id: 'startup_items', name: '过期启动菜单项', category: '软件', count: 0 },
    { id: 'startup_programs', name: '过期或无效的启动程序', category: '软件', count: 0 },
    { id: 'sound_events', name: '过期或无效的声音或事件设置', category: '系统', count: 0 },
];

export function RegistryCleaner() {
    const [items, setItems] = useState<RegistryItem[]>(REGISTRY_CATEGORIES);
    const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set(items.map(i => i.id)));
    const [isScanning, setIsScanning] = useState(false);
    const [isCleaning, setIsCleaning] = useState(false);
    const [lastScanTime, setLastScanTime] = useState<string | null>(null);
    const [status, setStatus] = useState<{ type: 'success' | 'error' | 'info', message: string } | null>(null);

    const toggleItem = (id: string) => {
        setSelectedIds(prev => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });
    };

    const selectAll = () => setSelectedIds(new Set(items.map(i => i.id)));
    const selectNone = () => setSelectedIds(new Set());

    const handleScan = async () => {
        setIsScanning(true);
        setStatus({ type: 'info', message: '正在扫描注册表...' });

        // Simulate scan - replace with actual backend call when implemented
        await new Promise(resolve => setTimeout(resolve, 1500));

        // Generate random counts for demonstration
        const scannedItems = items.map(item => ({
            ...item,
            count: Math.floor(Math.random() * 50)
        }));

        setItems(scannedItems);
        setLastScanTime(new Date().toLocaleString());
        setIsScanning(false);

        const totalIssues = scannedItems.reduce((sum, i) => sum + i.count, 0);
        setStatus({ type: 'success', message: `扫描完成，发现 ${totalIssues} 个问题` });
    };

    const handleClean = async () => {
        setIsCleaning(true);
        setStatus({ type: 'info', message: '正在清理...' });

        // Simulate clean - replace with actual backend call
        await new Promise(resolve => setTimeout(resolve, 1000));

        // Reset counts for cleaned items
        const cleanedItems = items.map(item => ({
            ...item,
            count: selectedIds.has(item.id) ? 0 : item.count
        }));

        setItems(cleanedItems);
        setIsCleaning(false);
        setStatus({ type: 'success', message: '清理完成！' });
    };

    const totalSelected = items.filter(i => selectedIds.has(i.id)).reduce((sum, i) => sum + i.count, 0);

    return (
        <div className="glass rounded-2xl p-6 shadow-soft flex flex-col">
            {/* Header */}
            <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-orange-500 to-red-500 flex items-center justify-center">
                        <Trash2 size={20} className="text-white" />
                    </div>
                    <div>
                        <h3 className="font-semibold text-slate-700">注册表清理</h3>
                        <p className="text-xs text-slate-400">
                            {lastScanTime ? `上次扫描: ${lastScanTime}` : '扫描并清理无效注册表项'}
                        </p>
                    </div>
                </div>
                <button
                    onClick={handleScan}
                    disabled={isScanning}
                    className="px-4 py-2 rounded-xl bg-gradient-to-r from-green-500 to-emerald-500 text-white font-medium text-sm flex items-center gap-2 hover:shadow-lg transition-all disabled:opacity-50"
                >
                    {isScanning ? (
                        <RefreshCw size={14} className="animate-spin" />
                    ) : (
                        <Search size={14} />
                    )}
                    {isScanning ? '扫描中...' : '开始扫描'}
                </button>
            </div>

            {/* Status */}
            {status && (
                <div className={`mb-3 px-3 py-2 rounded-lg text-xs flex items-center gap-2 ${status.type === 'success' ? 'bg-green-50 text-green-600' :
                        status.type === 'error' ? 'bg-red-50 text-red-600' :
                            'bg-blue-50 text-blue-600'
                    }`}>
                    {status.type === 'success' ? <Check size={14} /> :
                        status.type === 'info' ? <RefreshCw size={14} className="animate-spin" /> :
                            <AlertTriangle size={14} />}
                    {status.message}
                </div>
            )}

            {/* Items Grid */}
            <div className="flex-1 overflow-y-auto grid grid-cols-2 gap-2 max-h-[250px]">
                {items.map(item => (
                    <div
                        key={item.id}
                        onClick={() => toggleItem(item.id)}
                        className={`p-3 rounded-lg border cursor-pointer flex items-center justify-between ${selectedIds.has(item.id)
                                ? 'bg-orange-50/50 border-orange-200'
                                : 'bg-white/50 border-slate-100'
                            }`}
                    >
                        <div className="flex items-center gap-2">
                            <div className={`w-4 h-4 rounded border flex items-center justify-center ${selectedIds.has(item.id) ? 'bg-orange-500 border-orange-500' : 'bg-white border-slate-300'
                                }`}>
                                {selectedIds.has(item.id) && <Check size={10} className="text-white" />}
                            </div>
                            <span className={`text-xs font-medium ${selectedIds.has(item.id) ? 'text-orange-700' : 'text-slate-600'}`}>
                                {item.name}
                            </span>
                        </div>
                        {item.count > 0 && (
                            <span className="text-xs font-bold text-orange-500 bg-orange-100 px-2 py-0.5 rounded-full">
                                {item.count}
                            </span>
                        )}
                    </div>
                ))}
            </div>

            {/* Footer */}
            <div className="mt-4 pt-4 border-t flex items-center justify-between">
                <div className="flex items-center gap-2 text-xs text-slate-400">
                    <button onClick={selectAll} className="hover:text-blue-500 transition-colors">全选</button>
                    <span>·</span>
                    <button onClick={selectNone} className="hover:text-blue-500 transition-colors">全不选</button>
                </div>
                <button
                    onClick={handleClean}
                    disabled={isCleaning || totalSelected === 0}
                    className={`px-5 py-2.5 rounded-xl font-medium text-sm flex items-center gap-2 transition-all ${isCleaning || totalSelected === 0
                            ? 'bg-slate-100 text-slate-400'
                            : 'bg-gradient-to-r from-orange-500 to-red-500 text-white shadow-lg hover:shadow-xl'
                        }`}
                >
                    {isCleaning ? (
                        <RefreshCw size={14} className="animate-spin" />
                    ) : (
                        <Trash2 size={14} />
                    )}
                    {isCleaning ? '清理中...' : `立即清理 (${totalSelected})`}
                </button>
            </div>
        </div>
    );
}
