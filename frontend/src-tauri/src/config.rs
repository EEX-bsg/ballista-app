/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! アプリケーション設定管理モジュール
//! 
//! このモジュールは、アプリケーションの設定を管理するための機能を提供します。
//! 設定はJSONファイルとして保存され、アプリケーションの起動時に読み込まれます。

use log::{debug, error, info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::{Arc, Mutex}};
use tauri::AppHandle;

/// アプリケーション全体の設定を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub logging: LoggingConfig,
    pub modding: ModdingConfig,
    pub ui: UiConfig,
}

/// ロギング関連の設定を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    /// ログレベル（"trace", "debug", "info", "warn", "error", "off"）
    pub level: String,
    /// ファイルローテーション設定
    pub file_rotation: FileRotationConfig,
}

/// ログファイルのローテーション設定を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileRotationConfig {
    /// 保持する最大ファイル数
    pub max_files: usize,
    /// 各ログファイルの最大サイズ（MB）
    pub max_size_mb: u64,
}

/// MOD管理関連の設定を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModdingConfig {
    /// 保持するバックアップの数
    pub backup_count: usize,
    /// 自動バックアップを有効にするかどうか
    pub auto_backup: bool,
}

/// UI関連の設定を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    /// テーマ（"light", "dark", "system"）
    pub theme: String,
    /// 言語（"ja", "en"）
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig {
                level: "trace".to_string(),
                file_rotation: FileRotationConfig {
                    max_files: 10,
                    max_size_mb: 10,
                },
            },
            modding: ModdingConfig {
                backup_count: 10,
                auto_backup: true,
            },
            ui: UiConfig {
                theme: "light".to_string(),
                language: "ja".to_string(),
            },
        }
    }
}

/// アプリケーションの設定ディレクトリを取得する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
///
/// # 戻り値
///
/// 設定ディレクトリのパス。ディレクトリが存在しない場合は作成される。
pub fn get_config_dir(app_handle: &AppHandle) -> PathBuf {
    let app_dir = app_handle
        .path_resolver()
        .app_config_dir()
        .expect("Failed to get app config directory");
    
    // ディレクトリが存在しない場合は作成
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("Failed to create app config directory");
    }
    
    app_dir
}

/// 設定ファイルのパスを取得する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
///
/// # 戻り値
///
/// 設定ファイルのパス
pub fn get_config_file_path(app_handle: &AppHandle) -> PathBuf {
    get_config_dir(app_handle).join("config.json")
}

/// 設定ファイルを読み込む
///
/// 設定ファイルが存在しない場合やエラーが発生した場合は、デフォルト設定を返す
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
///
/// # 戻り値
///
/// アプリケーションの設定
pub fn load_config(app_handle: &AppHandle) -> AppConfig {
    let config_path = get_config_file_path(app_handle);
    
    if !config_path.exists() {
        let default_config = AppConfig::default();
        save_config(app_handle, &default_config);
        return default_config;
    }
    
    match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse config file: {}", e);
                let default_config = AppConfig::default();
                save_config(app_handle, &default_config);
                default_config
            }
        },
        Err(e) => {
            error!("Failed to read config file: {}", e);
            let default_config = AppConfig::default();
            save_config(app_handle, &default_config);
            default_config
        }
    }
}

/// 設定をファイルに保存する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
/// * `config` - 保存する設定
pub fn save_config(app_handle: &AppHandle, config: &AppConfig) {
    let config_path = get_config_file_path(app_handle);
    
    match serde_json::to_string_pretty(config) {
        Ok(json) => {
            if let Err(e) = fs::write(&config_path, json) {
                error!("Failed to write config file: {}", e);
            } else {
                debug!("Config file saved to: {}", config_path.display());
            }
        },
        Err(e) => error!("Failed to serialize config: {}", e),
    }
}

/// ログレベル文字列をLevelFilterに変換する
///
/// # 引数
///
/// * `level_str` - ログレベルを表す文字列 ("trace", "debug", "info", "warn", "error", "off")
///
/// # 戻り値
///
/// 対応するLevelFilter。不明な値の場合はLevelFilter::Infoを返す
pub fn get_log_level(level_str: &str) -> LevelFilter {
    match level_str.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "off" => LevelFilter::Off,
        _ => LevelFilter::Info,
    }
}

// グローバルな設定を保持する変数
pub static APP_CONFIG: once_cell::sync::Lazy<Arc<Mutex<Option<AppConfig>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// アプリケーションの設定を更新する
///
/// 設定ファイルを読み込み、グローバル設定変数を更新する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
pub fn update_config(app_handle: &AppHandle) {
    let config = load_config(app_handle);
    
    // グローバル設定を更新
    if let Ok(mut app_config) = APP_CONFIG.lock() {
        *app_config = Some(config.clone());
    }
    
    info!("設定を更新しました: ログレベル = {}", config.logging.level);
} 