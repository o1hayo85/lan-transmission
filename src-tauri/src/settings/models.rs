use serde::{Deserialize, Serialize};

/// 应用配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_save_path: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_save_path: String::new(),
        }
    }
}