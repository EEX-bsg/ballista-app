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

use crate::logger;
use crate::trace_fn;

/// アプリケーション全体の設定を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub path: PathConfig,
    pub logging: LoggingConfig,
    pub ui: UiConfig,
}

/// ファイル/ディレクトリパスの設定を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathConfig {
    /// Besiege.exeのパス
    pub besiege_path: String,
    /// Steam Workshopのディレクトリパス
    pub workshop_dir_path: String,
    /// Ballista appのデータ格納パス
    pub ballista_data_path: String,
}

/// ロギング関連の設定を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    /// ログレベル（"trace", "debug", "info", "warn", "error", "off"）
    pub level: String,
    /// 保持する最大ファイル数
    pub max_files: usize,
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
            path: PathConfig {
                besiege_path: "C:/Program Files (x86)/Steam/steamapps/common/Besiege/Besiege.exe".to_string(),
                workshop_dir_path: "C:/Program Files (x86)/Steam/steamapps/workshop/content/346010".to_string(),
                ballista_data_path: "$HOME/AppData/Roaming/ballista-app/".to_string(),
            },
            logging: LoggingConfig {
                level: "trace".to_string(),
                max_files: 10,
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
    trace_fn!("get_config_dir(app_handle: &AppHandle)");
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
    trace_fn!("get_config_file_path(app_handle: &AppHandle)");
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
    trace_fn!("load_config(app_handle: &AppHandle)");
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
    trace_fn!("save_config(app_handle: &AppHandle, config: {:?})", config);
    let config_path = get_config_file_path(app_handle);
    
    // カスタムフォーマッタを使用してスペース4つのインデントを適用
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);
    
    match config.serialize(&mut serializer) {
        Ok(_) => {
            match String::from_utf8(buf) {
                Ok(json) => {
                    if let Err(e) = fs::write(&config_path, json) {
                        error!("Failed to write config file: {}", e);
                    } else {
                        debug!("Config file saved to: {}", config_path.display());
                    }
                },
                Err(e) => error!("Failed to convert serialized config to UTF-8: {}", e),
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
pub fn string_to_log_level(level_str: &str) -> LevelFilter {
    trace_fn!("string_to_log_level(level_str: {})", level_str);
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

/// 現在のログレベルを取得する
///
/// # 戻り値
///
/// ログレベル（LevelFilter）。設定が読み込まれていない場合はデフォルト値を返す
pub fn get_log_level() -> LevelFilter {
    trace_fn!("get_log_level()");
    let level_str = get_config_value(
        |config| config.logging.level.clone(),
        AppConfig::default().logging.level
    );
    string_to_log_level(&level_str)
}

/// ログレベルを設定する
///
/// 指定されたログレベルに設定を更新し、設定ファイルに保存する
/// また、実際のロガーのログレベルも更新する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
/// * `level` - 設定するログレベル ("trace", "debug", "info", "warn", "error", "off")
///
/// # 戻り値
///
/// 設定の更新に成功した場合は`Ok(())`、失敗した場合は`Err`
pub fn set_log_level(app_handle: &AppHandle, level: &str) -> Result<(), String> {
    trace_fn!("set_log_level(app_handle: &AppHandle, level: {})", level);
    let level_lowercase = level.to_lowercase();
    
    // 有効なログレベルかチェック
    let log_filter = match level_lowercase.as_str() {
        "trace" | "debug" | "info" | "warn" | "error" | "off" => string_to_log_level(&level_lowercase),
        _ => return Err(BallistaError::ConfigError(format!("無効なログレベル: {}", level))),
    };
    
    update_config_value(
        app_handle,
        || Ok(()), // 検証は既に行っているので、ここでは常にOKを返す
        |config| {
            config.logging.level = level_lowercase.clone();
        },
        &format!("ログレベルを更新しました: {}", level_lowercase)
    )?;
    
    // 実際のロガーのログレベルも更新（これは他のsetter関数にはない特殊な処理）
    logger::set_log_level(log_filter);
    
    Ok(())
}

/// 設定値を取得するための汎用関数
///
/// # 型パラメータ
///
/// * `F` - 設定値を取得する関数の型
/// * `T` - 取得する設定値の型
///
/// # 引数
///
/// * `getter` - 設定値を取得する関数
/// * `default` - 設定が読み込まれていない場合のデフォルト値を取得する関数
///
/// # 戻り値
///
/// 設定値。設定が読み込まれていない場合はデフォルト値を返す
fn get_config_value<F, T>(getter: F, default: T) -> T
where
    F: FnOnce(&AppConfig) -> T,
    T: Clone,
{
    if let Ok(app_config) = APP_CONFIG.lock() {
        if let Some(config) = &*app_config {
            return getter(config);
        }
    }
    default
}

/// パスの正規化を行う
///
/// # 引数
///
/// * `path` - 正規化するパス
///
/// # 戻り値
///
/// 正規化されたパス
fn normalize_path(path: &str) -> String {
    trace_fn!("normalize_path(path: {})", path);
    path.trim().replace('\\', "/").to_string()
}

/// 設定値を更新するための汎用関数
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
/// * `validator` - 設定値を検証する関数
/// * `updater` - 設定値を更新する関数
/// * `log_message` - 更新成功時のログメッセージ
///
/// # 戻り値
///
/// 設定の更新に成功した場合は`Ok(())`、失敗した場合は`Err`
fn update_config_value<V, U>(
    app_handle: &AppHandle,
    validator: V,
    updater: U,
    log_message: &str,
) -> Result<(), BallistaError>
where
    V: FnOnce() -> Result<(), BallistaError>,
    U: FnOnce(&mut AppConfig),
{
    // 値を検証
    validator()?;
    
    // 現在の設定を取得
    let mut config = load_config(app_handle);
    
    // 設定値を更新
    updater(&mut config);
    
    // 設定ファイルに保存
    save_config(app_handle, &config);
    
    // グローバル設定を更新
    update_global_config(config.clone());
    
    info!("{}", log_message);
    Ok(())
}

/// Besiegeのパスを取得する
///
/// # 戻り値
///
/// Besiegeのパス。設定が読み込まれていない場合はデフォルト値を返す
pub fn get_besiege_path() -> String {
    trace_fn!("get_besiege_path()");
    get_config_value(
        |config| config.path.besiege_path.clone(),
        AppConfig::default().path.besiege_path
    )
}

/// Besiegeのパスを設定する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
/// * `path` - 設定するBesiegeのパス
///
/// # 戻り値
///
/// 設定の更新に成功した場合は`Ok(())`、失敗した場合は`Err`
pub fn set_besiege_path(app_handle: &AppHandle, path: &str) -> Result<(), BallistaError> {
    trace_fn!("set_besiege_path(path: {})", path);
    let path_normalized = normalize_path(path);
    trace_fn!("set_besiege_path(app_handle: &AppHandle, path: {})", path_normalized);
    
    update_config_value(
        app_handle,
        || {
            // パスが空でないことを確認
            if path_normalized.is_empty() {
                return Err(BallistaError::ConfigError("Besiegeのパスが空です".to_string()));
            }
            Ok(())
        },
        |config| {
            config.path.besiege_path = path_normalized.clone();
        },
        &format!("Besiegeのパスを更新しました: {}", path_normalized)
    )
}

/// Steam Workshopのパスを取得する
///
/// # 戻り値
///
/// Steam Workshopのパス。設定が読み込まれていない場合はデフォルト値を返す
pub fn get_workshop_path() -> String {
    trace_fn!("get_workshop_path()");
    get_config_value(
        |config| config.path.workshop_dir_path.clone(),
        AppConfig::default().path.workshop_dir_path
    )
}

/// Steam Workshopのパスを設定する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
/// * `path` - 設定するSteam Workshopのパス
///
/// # 戻り値
///
/// 設定の更新に成功した場合は`Ok(())`、失敗した場合は`Err`
pub fn set_workshop_path(app_handle: &AppHandle, path: &str) -> Result<(), BallistaError> {
    trace_fn!("set_workshop_path(path: {})", path);
    let path_normalized = normalize_path(path);
    trace_fn!("set_workshop_path(app_handle: &AppHandle, path: {})", path_normalized);
    
    update_config_value(
        app_handle,
        || {
            // パスが空でないことを確認
            if path_normalized.is_empty() {
                return Err(BallistaError::ConfigError("Steam Workshopのパスが空です".to_string()));
            }
            Ok(())
        },
        |config| {
            config.path.workshop_dir_path = path_normalized.clone();
        },
        &format!("Steam Workshopのパスを更新しました: {}", path_normalized)
    )
}

/// Ballistaデータパスを取得する
///
/// # 戻り値
///
/// Ballistaデータパス。設定が読み込まれていない場合はデフォルト値を返す
pub fn get_ballista_data_path() -> String {
    trace_fn!("get_ballista_data_path()");
    get_config_value(
        |config| config.path.ballista_data_path.clone(),
        AppConfig::default().path.ballista_data_path
    )
}

/// Ballistaデータパスを設定する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
/// * `path` - 設定するBallistaデータパス
///
/// # 戻り値
///
/// 設定の更新に成功した場合は`Ok(())`、失敗した場合は`Err`
pub fn set_ballista_data_path(app_handle: &AppHandle, path: &str) -> Result<(), BallistaError> {
    trace_fn!("set_ballista_data_path(path: {})", path);
    let path_normalized = normalize_path(path);
    trace_fn!("set_ballista_data_path(app_handle: &AppHandle, path: {})", path_normalized);
    
    update_config_value(
        app_handle,
        || {
            // パスが空でないことを確認
            if path_normalized.is_empty() {
                return Err(BallistaError::ConfigError("Ballistaデータパスが空です".to_string()));
            }
            Ok(())
        },
        |config| {
            config.path.ballista_data_path = path_normalized.clone();
        },
        &format!("Ballistaデータパスを更新しました: {}", path_normalized)
    )
}

/// UIテーマを取得する
///
/// # 戻り値
///
/// UIテーマ。設定が読み込まれていない場合はデフォルト値を返す
pub fn get_ui_theme() -> String {
    trace_fn!("get_ui_theme()");
    get_config_value(
        |config| config.ui.theme.clone(),
        AppConfig::default().ui.theme
    )
}

/// UIテーマを設定する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
/// * `theme` - 設定するUIテーマ ("light", "dark", "system")
///
/// # 戻り値
///
/// 設定の更新に成功した場合は`Ok(())`、失敗した場合は`Err`
pub fn set_ui_theme(app_handle: &AppHandle, theme: &str) -> Result<(), BallistaError> {
    trace_fn!("set_ui_theme(theme: {})", theme);
    let theme_lowercase = theme.to_lowercase();
    trace_fn!("set_ui_theme(app_handle: &AppHandle, theme: {})", theme_lowercase);
    
    update_config_value(
        app_handle,
        || {
            // 有効なテーマかチェック
            match theme_lowercase.as_str() {
                "light" | "dark" | "system" => Ok(()),
                _ => Err(BallistaError::ConfigError(format!("無効なUIテーマ: {}", theme))),
            }
        },
        |config| {
            config.ui.theme = theme_lowercase.clone();
        },
        &format!("UIテーマを更新しました: {}", theme_lowercase)
    )
}

/// 言語設定を取得する
///
/// # 戻り値
///
/// 言語設定。設定が読み込まれていない場合はデフォルト値を返す
pub fn get_language() -> String {
    trace_fn!("get_language()");
    get_config_value(
        |config| config.ui.language.clone(),
        AppConfig::default().ui.language
    )
}

/// 言語設定を設定する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
/// * `language` - 設定する言語 ("ja", "en")
///
/// # 戻り値
///
/// 設定の更新に成功した場合は`Ok(())`、失敗した場合は`Err`
pub fn set_language(app_handle: &AppHandle, language: &str) -> Result<(), BallistaError> {
    trace_fn!("set_language(language: {})", language);
    let language_lowercase = language.to_lowercase();
    trace_fn!("set_language(app_handle: &AppHandle, language: {})", language_lowercase);
    
    update_config_value(
        app_handle,
        || {
            // 有効な言語かチェック
            match language_lowercase.as_str() {
                "ja" | "en" => Ok(()),
                _ => Err(BallistaError::ConfigError(format!("無効な言語設定: {}", language))),
            }
        },
        |config| {
            config.ui.language = language_lowercase.clone();
        },
        &format!("言語設定を更新しました: {}", language_lowercase)
    )
}

// グローバルな設定を保持する変数
pub static APP_CONFIG: once_cell::sync::Lazy<Arc<Mutex<Option<AppConfig>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// グローバル設定を更新する
///
/// # 引数
///
/// * `config` - 更新する設定
fn update_global_config(config: AppConfig) {
    trace_fn!("update_global_config(config: {:?})", config);
    if let Ok(mut app_config) = APP_CONFIG.lock() {
        *app_config = Some(config);
    }
}

/// 現在の設定をログに出力する
///
/// # 引数
///
/// * `config` - ログに出力する設定
fn log_current_config(config: &AppConfig) {
    trace_fn!("log_current_config(config: {:?})", config);
    info!("設定を更新しました: ログレベル = {}, テーマ = {}, 言語 = {}", 
        config.logging.level, config.ui.theme, config.ui.language);
    info!("パス設定: Besiege = {}, Workshop = {}, Ballistaデータ = {}", 
        config.path.besiege_path, config.path.workshop_dir_path, config.path.ballista_data_path);
}

/// アプリケーションの設定を更新する
///
/// 設定ファイルを読み込み、グローバル設定変数を更新する
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
pub fn update_config(app_handle: &AppHandle) {
    trace_fn!("update_config(app_handle: &AppHandle)");
    let config = load_config(app_handle);
    
    // グローバル設定を更新
    update_global_config(config.clone());
    
    // 現在の設定をログに出力
    log_current_config(&config);
}
