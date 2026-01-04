import React, { useState, useEffect } from 'react';
import { Trash2, Check, AlertTriangle, Search, RefreshCw, Download, Upload, RotateCcw, Save } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

interface RegistryIssue {
    path: string;
    value_name: string | null;
    issue_type: string;
    details: string;
}

interface RegistryScanResult {
    category: string;
    count: number;
    items: RegistryIssue[];
}

export function RegistryCleaner() {
    const [scanResults, setScanResults] = useState<RegistryScanResult[]>([]);
    const [selectedCategories, setSelectedCategories] = useState<Set<string>>(new Set());
    const [isScanning, setIsScanning] = useState(false);
    const [isCleaning, setIsCleaning] = useState(false);
    const [isBackingUp, setIsBackingUp] = useState(false);
    const [lastScanTime, setLastScanTime] = useState<string | null>(null);
    const [backups, setBackups] = useState<string[]>([]);
    const [status, setStatus] = useState<{ type: 'success' | 'error' | 'info', message: string } | null>(null);

    // Load existing backups on mount
    useEffect(() => {
        loadBackups();
    }, []);

    const loadBackups = async () => {
        try {
            const list = await invoke<string[]>('list_registry_backups');
            setBackups(list);
        } catch (e) {
            console.error('Failed to load backups:', e);
        }
    };

    const toggleCategory = (category: string) => {
        setSelectedCategories(prev => {
            const next = new Set(prev);
            if (next.has(category)) next.delete(category);
            else next.add(category);
            return next;
        });
    };

    const selectAll = () => setSelectedCategories(new Set(scanResults.map(r => r.category)));
    const selectNone = () => setSelectedCategories(new Set());

    const handleScan = async () => {
        setIsScanning(true);
        setStatus({ type: 'info', message: '正在扫描注册表...' });

        try {
            const results = await invoke<RegistryScanResult[]>('scan_registry');
            setScanResults(results);
            setSelectedCategories(new Set(results.map(r => r.category)));
            setLastScanTime(new Date().toLocaleString());

            const totalIssues = results.reduce((sum, r) => sum + r.count, 0);
            setStatus({ type: 'success', message: `扫描完成，发现 ${totalIssues} 个问题` });
        } catch (e) {
            setStatus({ type: 'error', message: `扫描失败: ${e}` });
        } finally {
            setIsScanning(false);
        }
    };

    const handleClean = async () => {
        // Gather all issues from selected categories
        const issuesToClean: RegistryIssue[] = scanResults
            .filter(r => selectedCategories.has(r.category))
            .flatMap(r => r.items);

        if (issuesToClean.length === 0) {
            setStatus({ type: 'info', message: '没有需要清理的项目' });
            return;
        }

        setIsCleaning(true);
        setStatus({ type: 'info', message: '正在清理...' });

        try {
            const cleaned = await invoke<number>('clean_registry', { issues: issuesToClean });

            // Refresh scan results
            await handleScan();

            setStatus({ type: 'success', message: `成功清理 ${cleaned} 个注册表项` });
        } catch (e) {
            setStatus({ type: 'error', message: `清理失败: ${e}` });
        } finally {
            setIsCleaning(false);
        }
    };

    const handleBackup = async () => {
        setIsBackingUp(true);
        setStatus({ type: 'info', message: '正在备份注册表...' });

        try {
            const filename = await invoke<string>('create_full_backup');
            await loadBackups();
            setStatus({ type: 'success', message: `备份成功: ${filename}` });
        } catch (e) {
            setStatus({ type: 'error', message: `备份失败: ${e}` });
        } finally {
            setIsBackingUp(false);
        }
    };

    const handleExportBackup = async () => {
        const path = await save({
            filters: [{ name: 'Registry File', extensions: ['reg'] }],
            defaultPath: `registry_backup_${Date.now()}.reg`
        });

        if (path) {
            setStatus({ type: 'info', message: '正在导出...' });
            try {
                await invoke('backup_registry', {
                    path,
                    key: 'HKEY_CURRENT_USER\\Software'
                });
                setStatus({ type: 'success', message: '导出成功!' });
            } catch (e) {
                setStatus({ type: 'error', message: `导出失败: ${e}` });
            }
        }
    };

    const handleImportRestore = async () => {
        const selected = await open({
            filters: [{ name: 'Registry File', extensions: ['reg'] }],
            multiple: false
        });

        if (selected) {
            setStatus({ type: 'info', message: '正在导入注册表...' });
            try {
                await invoke('import_registry', { path: selected });
                setStatus({ type: 'success', message: '注册表还原成功!' });
            } catch (e) {
                setStatus({ type: 'error', message: `导入失败: ${e}` });
            }
        }
    };

    const totalIssues = scanResults.reduce((sum, r) => sum + r.count, 0);
    const selectedIssues = scanResults
        .filter(r => selectedCategories.has(r.category))
        .reduce((sum, r) => sum + r.count, 0);

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

            {/* Backup/Restore Toolbar */}
            <div className="flex gap-2 mb-4">
                <button
                    onClick={handleBackup}
                    disabled={isBackingUp}
                    className="flex-1 px-3 py-2 rounded-lg border border-slate-200 text-slate-600 hover:bg-slate-50 text-xs font-medium flex items-center justify-center gap-1.5 transition-colors"
                >
                    <Save size={12} />
                    {isBackingUp ? '备份中...' : '备份注册表'}
                </button>
                <button
                    onClick={handleExportBackup}
                    className="flex-1 px-3 py-2 rounded-lg border border-slate-200 text-slate-600 hover:bg-slate-50 text-xs font-medium flex items-center justify-center gap-1.5 transition-colors"
                >
                    <Download size={12} />
                    导出
                </button>
                <button
                    onClick={handleImportRestore}
                    className="flex-1 px-3 py-2 rounded-lg border border-slate-200 text-slate-600 hover:bg-slate-50 text-xs font-medium flex items-center justify-center gap-1.5 transition-colors"
                >
                    <Upload size={12} />
                    导入/还原
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

            {/* Scan Results */}
            <div className="flex-1 overflow-y-auto space-y-2 max-h-[220px]">
                {scanResults.length === 0 ? (
                    <div className="text-center text-slate-400 py-8 text-sm">
                        点击"开始扫描"检测注册表问题
                    </div>
                ) : (
                    scanResults.map(result => (
                        <div
                            key={result.category}
                            onClick={() => toggleCategory(result.category)}
                            className={`p-3 rounded-lg border cursor-pointer flex items-center justify-between ${selectedCategories.has(result.category)
                                    ? 'bg-orange-50/50 border-orange-200'
                                    : 'bg-white/50 border-slate-100'
                                }`}
                        >
                            <div className="flex items-center gap-2">
                                <div className={`w-4 h-4 rounded border flex items-center justify-center ${selectedCategories.has(result.category) ? 'bg-orange-500 border-orange-500' : 'bg-white border-slate-300'
                                    }`}>
                                    {selectedCategories.has(result.category) && <Check size={10} className="text-white" />}
                                </div>
                                <span className={`text-sm font-medium ${selectedCategories.has(result.category) ? 'text-orange-700' : 'text-slate-600'}`}>
                                    {result.category}
                                </span>
                            </div>
                            <span className={`text-xs font-bold px-2 py-0.5 rounded-full ${result.count > 0
                                    ? 'text-orange-500 bg-orange-100'
                                    : 'text-green-500 bg-green-100'
                                }`}>
                                {result.count}
                            </span>
                        </div>
                    ))
                )}
            </div>

            {/* Recent Backups */}
            {backups.length > 0 && (
                <div className="mt-3 pt-3 border-t">
                    <div className="text-xs text-slate-400 mb-1">最近备份:</div>
                    <div className="flex flex-wrap gap-1">
                        {backups.slice(0, 3).map(b => (
                            <span key={b} className="text-xs bg-slate-100 px-2 py-0.5 rounded text-slate-500">
                                {b}
                            </span>
                        ))}
                    </div>
                </div>
            )}

            {/* Footer */}
            <div className="mt-4 pt-4 border-t flex items-center justify-between">
                <div className="flex items-center gap-2 text-xs text-slate-400">
                    <button onClick={selectAll} className="hover:text-blue-500 transition-colors">全选</button>
                    <span>·</span>
                    <button onClick={selectNone} className="hover:text-blue-500 transition-colors">全不选</button>
                    <span className="ml-2 text-slate-300">|</span>
                    <span className="ml-2">共 {totalIssues} 项</span>
                </div>
                <button
                    onClick={handleClean}
                    disabled={isCleaning || selectedIssues === 0}
                    className={`px-5 py-2.5 rounded-xl font-medium text-sm flex items-center gap-2 transition-all ${isCleaning || selectedIssues === 0
                            ? 'bg-slate-100 text-slate-400'
                            : 'bg-gradient-to-r from-orange-500 to-red-500 text-white shadow-lg hover:shadow-xl'
                        }`}
                >
                    {isCleaning ? (
                        <RefreshCw size={14} className="animate-spin" />
                    ) : (
                        <Trash2 size={14} />
                    )}
                    {isCleaning ? '清理中...' : `立即清理 (${selectedIssues})`}
                </button>
            </div>
        </div>
    );
}
