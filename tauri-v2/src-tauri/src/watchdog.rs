use crate::{config, governor, PriorityLevel, ProcessInfo};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashSet;
// use std::sync::atomic::{AtomicBool, Ordering};

// Track which processes we have restrained so we can restore them
static RESTRAINED_PIDS: Lazy<RwLock<HashSet<u32>>> = Lazy::new(|| RwLock::new(HashSet::new()));

// Debounce/Cool-down logic (prevent rapid toggling)
// static LAST_ACTION_TIME: Lazy<RwLock<std::time::Instant>> =
//     Lazy::new(|| RwLock::new(std::time::Instant::now()));

static LAST_TRIM_TIME: Lazy<RwLock<std::time::Instant>> =
    Lazy::new(|| RwLock::new(std::time::Instant::now() - std::time::Duration::from_secs(3600)));

pub async fn check_and_trim_memory() {
    let config = config::get_config().await.unwrap_or_default();
    let trim_config = config.smart_trim;

    if !trim_config.enabled {
        return;
    }

    // Check interval (default 30s)
    let last_trim = LAST_TRIM_TIME.read();
    if last_trim.elapsed().as_secs() < trim_config.interval as u64 {
        return;
    }
    drop(last_trim);

    // Get current memory status
    if let Ok(mem) = crate::hardware::get_memory_info().await {
        if mem.percent >= trim_config.threshold {
            tracing::info!(
                "Smart Trim: Memory usage {}% exceeds threshold {}%. Triggering optimization...",
                mem.percent,
                trim_config.threshold
            );
            
            // Perform cleanup
            let _ = governor::clear_system_memory().await;
            
            // Update last trim time
            let mut last_trim = LAST_TRIM_TIME.write();
            *last_trim = std::time::Instant::now();
        }
    }
}

pub async fn enforce_profiles(processes: &[ProcessInfo]) {
    let profiles = config::get_profiles().await.unwrap_or_default();
    if profiles.is_empty() {
        return;
    }

    for p in processes {
        let name_lower = p.name.to_lowercase();
        // Find a matching profile (case-insensitive)
        if let Some(profile) = profiles.iter().find(|pr| pr.name.to_lowercase() == name_lower && pr.enabled) {
            
            // 1. Check Priority
            if p.priority != profile.priority {
                if let Some(level) = PriorityLevel::from_str(&profile.priority) {
                    tracing::info!("Auto-Apply: Adjusting priority for {} (PID {}) to {}", p.name, p.pid, profile.priority);
                    let _ = governor::set_priority(p.pid, level).await;
                }
            }

            // 2. Check Affinity/Sets
            // This is harder to check perfectly because p.cpu_affinity is a formatted string.
            // But if it's different enough, we re-apply.
            // For now, let's look at the mode.
            let is_soft = profile.mode == "soft";

            // If profile is soft but current is not reported as "Sets: ..." 
            // OR if profile is hard but current doesn't match hex
            let needs_affinity_fix = if is_soft {
                !p.cpu_affinity.starts_with("Sets")
            } else {
                // Hard affinity
                let target_val = u64::from_str_radix(&profile.affinity, 16).unwrap_or(0);
                let target_hex = format!("{:#x}", target_val);
                p.cpu_affinity != target_hex && p.cpu_affinity != "All"
            };

            if needs_affinity_fix {
                tracing::info!("Auto-Apply: Re-applying affinity for {} (PID {}) [Mode: {}]", p.name, p.pid, profile.mode);
                if is_soft {
                    // Convert affinity string (core indices) back to Vec<u32>
                    // Note: In my Auto-Save implementation, I saved maskString to affinity.
                    // We need to be consistent. 
                    // Actually, let's check how handleAffinityApply saved it.
                    // It saved maskString.
                    
                    // For Sets, it's better to store core_ids. 
                    // But our struct has a single `affinity: String`.
                    // Let's assume for 'soft' mode, we might need a parser.
                    
                    // Actually, I'll just use the set_process_affinity for both if I can, 
                    // but Sets needs the cpu_sets module.
                    
                    if let Ok(mask) = u64::from_str_radix(&profile.affinity, 16) {
                        let mut core_ids = Vec::new();
                        for i in 0..64 {
                            if (mask & (1 << i)) != 0 {
                                core_ids.push(i as u32);
                            }
                        }
                        let _ = crate::cpu_sets::set_process_cpu_sets(p.pid, core_ids);
                    }
                } else {
                    let _ = governor::set_process_affinity(p.pid, profile.affinity.clone()).await;
                }
            }
        }
    }
}

pub async fn check_and_restrain(processes: &[ProcessInfo]) {
    // 1. Get Config
    let config = config::get_config().await.unwrap_or_default();
    let pb_config = config.pro_balance;

    if !pb_config.enabled {
        // If disabled, restore any restrained processes immediately
        restore_all().await;
        return;
    }

    // 2. Identify if any game is running (matching UI "当游戏运行时")
    let game_list: Vec<String> = config.game_list.iter().map(|s| s.to_lowercase()).collect();
    let is_any_game_running = processes.iter().any(|p| {
        let name_lower = p.name.to_lowercase();
        game_list.iter().any(|g| name_lower.contains(g))
    });

    if !is_any_game_running {
        // Only restrain when a game is detected
        restore_all().await;
        return;
    }

    // 3. Calculate Total System Load
    let system = sysinfo::System::new_all();
    let logical_cores = system.cpus().len() as f32;
    let total_cpu_sum: f32 = processes.iter().map(|p| p.cpu_usage).sum();
    let total_cpu_percent = if logical_cores > 0.0 {
        total_cpu_sum / logical_cores
    } else {
        0.0
    };

    // 4. Logic
    let threshold = pb_config.cpu_threshold;

    if total_cpu_percent > threshold {
        // High Load while Gaming - Find background culprits
        restrain_processes(processes, &pb_config.excluded_processes, &game_list).await;
    } else {
        // Normal Load - Restore
        restore_all().await;
    }
}

async fn restrain_processes(processes: &[ProcessInfo], excludes: &[String], games: &[String]) {
    let mut restrained = RESTRAINED_PIDS.write();
    let foreground_pid = governor::get_foreground_window_pid().unwrap_or(0);

    for p in processes {
        let name_lower = p.name.to_lowercase();

        // Criteria to Restrain:
        // 1. Not already Idle/BelowNormal
        let current_pri = &p.priority;
        let is_target_pri =
            current_pri == "Normal" || current_pri == "AboveNormal" || current_pri == "High";

        if !is_target_pri {
            continue;
        }

        // 2. Not Foreground
        if p.pid == foreground_pid {
            continue;
        }

        // 3. Not in Game List (Games should never be suppressed)
        if games.iter().any(|g| name_lower.contains(g)) {
            continue;
        }

        // 4. Not Excluded (Manual exclusion)
        if excludes
            .iter()
            .any(|ex| name_lower.contains(&ex.to_lowercase()))
        {
            continue;
        }

        // 5. Using significant CPU?
        if p.cpu_usage < 1.0 {
            continue;
        }

        // ACT: Restrain
        if !restrained.contains(&p.pid) {
            tracing::info!(
                "ProBalance: Restraining Background process PID {} ({}) - CPU: {}",
                p.pid,
                p.name,
                p.cpu_usage
            );
            if let Ok(_) = governor::set_priority(p.pid, PriorityLevel::BelowNormal).await {
                restrained.insert(p.pid);
            }
        }
    }
}

async fn restore_all() {
    let mut restrained = RESTRAINED_PIDS.write();
    if restrained.is_empty() {
        return;
    }

    tracing::info!("ProBalance: Restoring {} processes", restrained.len());

    let pids: Vec<u32> = restrained.drain().collect();
    for pid in pids {
        // Restore to Normal (Default).
        // Ideal: Restore to original. But we didn't store it.
        // Most apps are Normal.
        let _ = governor::set_priority(pid, PriorityLevel::Normal).await;
    }
}
pub async fn apply_default_rules(processes: &[ProcessInfo]) {
    let config = config::get_config().await.unwrap_or_default();
    let rules = config.default_rules;

    if !rules.enabled {
        return;
    }

    let profiles = config::get_profiles().await.unwrap_or_default();
    let game_list: Vec<String> = config.game_list.iter().map(|s| s.to_lowercase()).collect();

    for p in processes {
        let name_lower = p.name.to_lowercase();

        // 1. 跳过已经有特定 Profile 的进程
        if profiles.iter().any(|pr| pr.name.to_lowercase() == name_lower && pr.enabled) {
            continue;
        }

        // 2. 判定是否在游戏列表中
        let is_game = game_list.iter().any(|g| name_lower.contains(g));

        // 3. 应用规则
        if is_game {
            // 应用游戏规则 (P-Core/CCD0)
            if let Some(mask) = &rules.game_mask {
                if p.cpu_affinity != *mask && p.cpu_affinity != format!("{:#x}", u64::from_str_radix(mask, 16).unwrap_or(0)) {
                    tracing::info!("DefaultRules: Mapping game {} to {}", p.name, mask);
                    let _ = governor::set_process_affinity(p.pid, mask.clone()).await;
                }
            }
            if let Some(level) = PriorityLevel::from_str(&rules.game_priority) {
                if p.priority != rules.game_priority {
                    let _ = governor::set_priority(p.pid, level).await;
                }
            }
        } else {
            // 应用系统/背景规则 (E-Core/CCD1)
            // 跳过核心进程
            if name_lower == "explorer.exe" || name_lower == "task-nexus.exe" || name_lower == "system" {
                continue;
            }

            if let Some(mask) = &rules.system_mask {
                if p.cpu_affinity != *mask && p.cpu_affinity != format!("{:#x}", u64::from_str_radix(mask, 16).unwrap_or(0)) {
                    // 仅对有一定负载或特定优先级的背景进程应用，避免对所有空闲进程操作
                    if p.cpu_usage > 0.1 || p.priority != "Normal" {
                        let _ = governor::set_process_affinity(p.pid, mask.clone()).await;
                    }
                }
            }
            if let Some(level) = PriorityLevel::from_str(&rules.system_priority) {
                if p.priority != rules.system_priority {
                    let _ = governor::set_priority(p.pid, level).await;
                }
            }
        }
    }
}
