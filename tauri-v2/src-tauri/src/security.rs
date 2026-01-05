use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

// ============================================================================
// Time Bomb Logic
// ============================================================================

// 预设截止日期：2026年2月1日
const EXPIRATION_DATE: (i32, u32, u32) = (2026, 2, 1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBombStatus {
    pub is_expired: bool,
    pub expiration_date: String,
    pub current_date: String,
    pub days_remaining: i64,
    pub verification_source: String, // "Network" or "System"
}

/// 检查内测版是否已过期
/// 策略：
/// 1. 尝试获取网络时间 (简单 HTTP HEAD 请求) - 暂未实现，为避免引入 heavy dependencies，先用系统时间
/// 2. 回退到系统时间
/// 3. 如果当前时间 > 截止日期，返回过期
pub async fn check_expiration() -> TimeBombStatus {
    // 构建截止日期
    let expiry = NaiveDate::from_ymd_opt(EXPIRATION_DATE.0, EXPIRATION_DATE.1, EXPIRATION_DATE.2)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    // 1. 获取当前时间 (目前仅使用系统时间，生产环境应加上网络时间校验)
    let now_local = Local::now().naive_local();
    
    // TODO: 实现简单的网络时间校验 (可选)
    // 如果需要更强的防篡改，可以请求 google.com 或百度
    // 但考虑到这只是防止普通用户长期使用旧版本，系统时间通常足够
    
    let is_expired = now_local > expiry;
    let duration = expiry.signed_duration_since(now_local);
    let days_remaining = duration.num_days();

    TimeBombStatus {
        is_expired,
        expiration_date: expiry.format("%Y-%m-%d").to_string(),
        current_date: now_local.format("%Y-%m-%d %H:%M:%S").to_string(),
        days_remaining: if is_expired { 0 } else { days_remaining },
        verification_source: "System".to_string(),
    }
}

// ============================================================================
// Data Encryption & License (Mock for now to fix build)
// ============================================================================

pub fn encrypt_data(data: &str) -> crate::AppResult<String> {
    // TODO: Implement actual AES-256 encryption
    // For now, just base64 encode to simulate obfuscation
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    Ok(STANDARD.encode(data))
}

pub fn decrypt_data(data: &str) -> crate::AppResult<String> {
    // TODO: Implement actual AES-256 decryption
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    match STANDARD.decode(data) {
        Ok(bytes) => String::from_utf8(bytes).map_err(|e| crate::AppError::SystemError(e.to_string())),
        Err(e) => Err(crate::AppError::SystemError(e.to_string())),
    }
}

pub fn get_machine_code() -> String {
    // Simple mock machine code
    "TASK-NEXUS-DEV-MACHINE".to_string()
}

pub fn verify_license(_key: &str) -> bool {
    // Simplified verification for dev
    true
}

pub async fn check_activation_status() -> bool {
    // Always active in dev
    true
}
