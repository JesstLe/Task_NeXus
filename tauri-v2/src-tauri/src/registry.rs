// Registry operations module
// Provides backup, restore, import, and cleaning functionality

use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use std::os::windows::process::CommandExt;
use winreg::enums::*;
use winreg::RegKey;
use serde::{Serialize, Deserialize};

use crate::{AppError, AppResult};

/// Registry scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryScanResult {
    pub category: String,
    pub count: u32,
    pub items: Vec<RegistryIssue>,
}

/// Single registry issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIssue {
    pub path: String,
    pub value_name: Option<String>,
    pub issue_type: String,
    pub details: String,
}

/// Backup registry to a .reg file using reg.exe
/// 
/// # Arguments
/// * `path` - Output file path (should end with .reg)
/// * `key` - Registry key to backup (e.g., "HKEY_CURRENT_USER\\Software")
#[tauri::command]
pub async fn backup_registry(path: String, key: String) -> Result<(), String> {
    backup_registry_internal(&path, &key)
        .await
        .map_err(|e| e.to_string())
}

async fn backup_registry_internal(path: &str, key: &str) -> AppResult<()> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::SystemError(format!("Failed to create directory: {}", e)))?;
    }

    // Use reg.exe for backup
    let output = Command::new("reg")
        .args(["export", key, path, "/y"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| AppError::SystemError(format!("Failed to execute reg.exe: {}", e)))?;

    if output.status.success() {
        tracing::info!("Registry backup successful: {} -> {}", key, path);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::SystemError(format!("Registry backup failed: {}", stderr)))
    }
}

/// Import a .reg file into registry
#[tauri::command]
pub async fn import_registry(path: String) -> Result<(), String> {
    import_registry_internal(&path)
        .await
        .map_err(|e| e.to_string())
}

async fn import_registry_internal(path: &str) -> AppResult<()> {
    if !Path::new(path).exists() {
        return Err(AppError::SystemError(format!("File not found: {}", path)));
    }

    // Use reg.exe for import
    let output = Command::new("reg")
        .args(["import", path])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| AppError::SystemError(format!("Failed to execute reg.exe: {}", e)))?;

    if output.status.success() {
        tracing::info!("Registry import successful: {}", path);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::SystemError(format!("Registry import failed: {}", stderr)))
    }
}

/// Restore registry from a backup file (same as import)
#[tauri::command]
pub async fn restore_registry(path: String) -> Result<(), String> {
    import_registry(path).await
}

/// Scan registry for invalid entries
#[tauri::command]
pub async fn scan_registry() -> Result<Vec<RegistryScanResult>, String> {
    scan_registry_internal()
        .await
        .map_err(|e| e.to_string())
}

async fn scan_registry_internal() -> AppResult<Vec<RegistryScanResult>> {
    let mut results = Vec::new();

    // 1. Check invalid uninstall entries
    let uninstall_issues = scan_uninstall_entries().await?;
    results.push(RegistryScanResult {
        category: "无效的卸载信息".to_string(),
        count: uninstall_issues.len() as u32,
        items: uninstall_issues,
    });

    // 2. Check invalid startup entries
    let startup_issues = scan_startup_entries().await?;
    results.push(RegistryScanResult {
        category: "无效的启动项".to_string(),
        count: startup_issues.len() as u32,
        items: startup_issues,
    });

    // 3. Check invalid file associations
    let assoc_issues = scan_file_associations().await?;
    results.push(RegistryScanResult {
        category: "无效的文件关联".to_string(),
        count: assoc_issues.len() as u32,
        items: assoc_issues,
    });

    // 4. Check invalid shared DLLs
    let dll_issues = scan_shared_dlls().await?;
    results.push(RegistryScanResult {
        category: "无效的共享DLL".to_string(),
        count: dll_issues.len() as u32,
        items: dll_issues,
    });

    Ok(results)
}

/// Scan HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall for invalid entries
async fn scan_uninstall_entries() -> AppResult<Vec<RegistryIssue>> {
    let mut issues = Vec::new();

    // Scan both 32-bit and 64-bit uninstall keys
    let paths = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];

    for (hkey, path) in paths {
        if let Ok(uninstall_key) = RegKey::predef(hkey).open_subkey(path) {
            for subkey_name in uninstall_key.enum_keys().filter_map(|x| x.ok()) {
                if let Ok(subkey) = uninstall_key.open_subkey(&subkey_name) {
                    // Check UninstallString
                    if let Ok(uninstall_string) = subkey.get_value::<String, _>("UninstallString") {
                        let exe_path = extract_path_from_command(&uninstall_string);
                        if !exe_path.is_empty() && !Path::new(&exe_path).exists() {
                            issues.push(RegistryIssue {
                                path: format!("{}\\{}\\{}", 
                                    if hkey == HKEY_LOCAL_MACHINE { "HKLM" } else { "HKCU" },
                                    path, subkey_name),
                                value_name: Some("UninstallString".to_string()),
                                issue_type: "invalid_path".to_string(),
                                details: format!("指向不存在的文件: {}", exe_path),
                            });
                        }
                    }

                    // Check InstallLocation
                    if let Ok(install_location) = subkey.get_value::<String, _>("InstallLocation") {
                        if !install_location.is_empty() && !Path::new(&install_location).exists() {
                            issues.push(RegistryIssue {
                                path: format!("{}\\{}\\{}", 
                                    if hkey == HKEY_LOCAL_MACHINE { "HKLM" } else { "HKCU" },
                                    path, subkey_name),
                                value_name: Some("InstallLocation".to_string()),
                                issue_type: "invalid_path".to_string(),
                                details: format!("安装目录不存在: {}", install_location),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(issues)
}

/// Scan startup entries (Run keys) for invalid paths
async fn scan_startup_entries() -> AppResult<Vec<RegistryIssue>> {
    let mut issues = Vec::new();

    let paths = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run"),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run"),
    ];

    for (hkey, path) in paths {
        if let Ok(run_key) = RegKey::predef(hkey).open_subkey(path) {
            for (name, value) in run_key.enum_values().filter_map(|x| x.ok()) {
                let command = value.to_string();
                let exe_path = extract_path_from_command(&command);
                if !exe_path.is_empty() && !Path::new(&exe_path).exists() {
                    issues.push(RegistryIssue {
                        path: format!("{}\\{}", 
                            if hkey == HKEY_LOCAL_MACHINE { "HKLM" } else { "HKCU" },
                            path),
                        value_name: Some(name),
                        issue_type: "invalid_startup".to_string(),
                        details: format!("启动项指向不存在的文件: {}", exe_path),
                    });
                }
            }
        }
    }

    Ok(issues)
}

/// Scan file associations for invalid references
async fn scan_file_associations() -> AppResult<Vec<RegistryIssue>> {
    let mut issues = Vec::new();

    // Only scan a subset for performance
    let classes_root = RegKey::predef(HKEY_CLASSES_ROOT);
    
    // Check a limited number of common extensions
    let test_extensions = [".txt", ".doc", ".pdf", ".exe", ".dll", ".jpg", ".png"];
    
    for ext in test_extensions {
        if let Ok(ext_key) = classes_root.open_subkey(ext) {
            if let Ok(prog_id) = ext_key.get_value::<String, _>("") {
                // Check if the ProgID exists
                if !prog_id.is_empty() {
                    if classes_root.open_subkey(&prog_id).is_err() {
                        issues.push(RegistryIssue {
                            path: format!("HKCR\\{}", ext),
                            value_name: Some("(Default)".to_string()),
                            issue_type: "invalid_association".to_string(),
                            details: format!("文件关联指向不存在的ProgID: {}", prog_id),
                        });
                    }
                }
            }
        }
    }

    Ok(issues)
}

/// Scan shared DLLs for invalid references
async fn scan_shared_dlls() -> AppResult<Vec<RegistryIssue>> {
    let mut issues = Vec::new();

    let path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\SharedDLLs";
    
    if let Ok(shared_dlls) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path) {
        // Limit scan to prevent excessive processing
        let mut count = 0;
        for (dll_path, _value) in shared_dlls.enum_values().filter_map(|x| x.ok()) {
            if count > 100 { break; } // Limit for performance
            
            if !Path::new(&dll_path).exists() {
                issues.push(RegistryIssue {
                    path: format!("HKLM\\{}", path),
                    value_name: Some(dll_path.clone()),
                    issue_type: "invalid_dll".to_string(),
                    details: format!("共享DLL不存在: {}", dll_path),
                });
            }
            count += 1;
        }
    }

    Ok(issues)
}

/// Clean specific registry issues
#[tauri::command]
pub async fn clean_registry(issues: Vec<RegistryIssue>) -> Result<u32, String> {
    clean_registry_internal(issues)
        .await
        .map_err(|e| e.to_string())
}

async fn clean_registry_internal(issues: Vec<RegistryIssue>) -> AppResult<u32> {
    let mut cleaned = 0u32;

    for issue in issues {
        let result = match issue.issue_type.as_str() {
            "invalid_path" | "invalid_startup" | "invalid_dll" => {
                // For these, we delete the value or subkey
                delete_registry_item(&issue.path, issue.value_name.as_deref()).await
            }
            "invalid_association" => {
                // Skip file associations for safety
                continue;
            }
            _ => continue,
        };

        if result.is_ok() {
            cleaned += 1;
            tracing::info!("Cleaned registry entry: {:?}", issue);
        } else {
            tracing::warn!("Failed to clean: {:?}", issue);
        }
    }

    Ok(cleaned)
}

/// Delete a registry key or value
async fn delete_registry_item(path: &str, value_name: Option<&str>) -> AppResult<()> {
    // Parse the path
    let parts: Vec<&str> = path.splitn(2, '\\').collect();
    if parts.len() < 2 {
        return Err(AppError::SystemError("Invalid registry path".to_string()));
    }

    let hkey = match parts[0] {
        "HKLM" => HKEY_LOCAL_MACHINE,
        "HKCU" => HKEY_CURRENT_USER,
        "HKCR" => HKEY_CLASSES_ROOT,
        _ => return Err(AppError::SystemError("Unknown registry hive".to_string())),
    };

    let subpath = parts[1];

    // Open with write access
    match RegKey::predef(hkey).open_subkey_with_flags(subpath, KEY_WRITE) {
        Ok(key) => {
            if let Some(value) = value_name {
                key.delete_value(value)
                    .map_err(|e| AppError::SystemError(format!("Failed to delete value: {}", e)))?;
            }
            Ok(())
        }
        Err(_) => {
            // If we can't open for write, try deleting the entire subkey
            if value_name.is_none() {
                // Find parent key and delete the subkey
                if let Some(last_sep) = subpath.rfind('\\') {
                    let parent_path = &subpath[..last_sep];
                    let subkey_name = &subpath[last_sep + 1..];
                    
                    if let Ok(parent) = RegKey::predef(hkey).open_subkey_with_flags(parent_path, KEY_WRITE) {
                        parent.delete_subkey_all(subkey_name)
                            .map_err(|e| AppError::SystemError(format!("Failed to delete subkey: {}", e)))?;
                        return Ok(());
                    }
                }
            }
            Err(AppError::SystemError("Failed to access registry key".to_string()))
        }
    }
}

/// Extract executable path from a command string
fn extract_path_from_command(command: &str) -> String {
    let command = command.trim();
    
    // Handle quoted paths
    if command.starts_with('"') {
        if let Some(end) = command[1..].find('"') {
            return command[1..end + 1].to_string();
        }
    }
    
    // Handle paths with MsiExec or other installers
    if command.to_lowercase().contains("msiexec") {
        return String::new(); // Skip MSI commands, they're typically valid
    }
    
    // Handle simple paths (space-delimited)
    if let Some(space) = command.find(' ') {
        let path = &command[..space];
        if path.contains('\\') || path.contains('/') {
            return path.to_string();
        }
    }
    
    // If no space, check if the whole thing is a path
    if command.contains('\\') || command.contains('/') {
        // Check if it ends with .exe
        if command.to_lowercase().ends_with(".exe") {
            return command.to_string();
        }
    }
    
    command.to_string()
}

/// Get list of available backup files
#[tauri::command]
pub async fn list_registry_backups() -> Result<Vec<String>, String> {
    let backup_dir = get_backup_directory();
    
    let mut backups = Vec::new();
    if let Ok(entries) = fs::read_dir(&backup_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "reg") {
                if let Some(name) = path.file_name() {
                    backups.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    
    Ok(backups)
}

/// Get default backup directory
fn get_backup_directory() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("com.tasknexus.app");
    path.push("registry_backups");
    path
}

/// Create a timestamped backup of important registry keys
#[tauri::command]
pub async fn create_full_backup() -> Result<String, String> {
    let backup_dir = get_backup_directory();
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("backup_{}.reg", timestamp);
    let full_path = backup_dir.join(&filename);
    
    // Backup the most important keys
    backup_registry(
        full_path.to_string_lossy().to_string(),
        "HKEY_CURRENT_USER\\Software".to_string()
    ).await?;
    
    Ok(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path() {
        assert_eq!(
            extract_path_from_command(r#""C:\Program Files\App\app.exe" --arg"#),
            r"C:\Program Files\App\app.exe"
        );
        assert_eq!(
            extract_path_from_command(r"C:\Windows\System32\cmd.exe /c"),
            r"C:\Windows\System32\cmd.exe"
        );
    }
}
