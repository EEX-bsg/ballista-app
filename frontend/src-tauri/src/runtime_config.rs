/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! ランタイム設定管理モジュール
//!
//! このモジュールは、アプリケーション実行中のみ保持するグローバル設定を管理するための機能を提供します。
//! これらの設定はメモリ上にのみ存在し、アプリケーションが終了すると消えます。

use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::trace_fn;
use crate::error::BallistaError;

/// ランタイム設定を表す構造体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuntimeConfig {
    /// 現在のセッションID
    pub session_id: String,
    /// 汎用的な設定値を保持するマップ
    pub values: HashMap<String, String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            values: HashMap::new(),
        }
    }
}

// グローバルなランタイム設定を保持する変数
pub static RUNTIME_CONFIG: once_cell::sync::Lazy<Arc<Mutex<Option<RuntimeConfig>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Some(RuntimeConfig::default()))));

/// ランタイム設定を初期化する
pub fn initialize_runtime_config() {
    trace_fn!("initialize_runtime_config()");
    let config = RuntimeConfig::default();
    update_global_runtime_config(config.clone());
    info!("ランタイム設定を初期化しました: セッションID = {}", config.session_id);
}

/// グローバルランタイム設定を更新する
///
/// # 引数
///
/// * `config` - 更新する設定
fn update_global_runtime_config(config: RuntimeConfig) {
    trace_fn!("update_global_runtime_config()");
    if let Ok(mut runtime_config) = RUNTIME_CONFIG.lock() {
        *runtime_config = Some(config);
    }
}

/// 現在のランタイム設定を取得する
///
/// # 戻り値
///
/// 現在のランタイム設定。設定が読み込まれていない場合はデフォルト値を返す
pub fn get_runtime_config() -> Option<RuntimeConfig> {
    trace_fn!("get_runtime_config()");
    if let Ok(runtime_config) = RUNTIME_CONFIG.lock() {
        return runtime_config.clone();
    }
    None
}

/// ランタイム設定をリセットする
pub fn reset_runtime_config() -> Result<(), BallistaError> {
    trace_fn!("reset_runtime_config()");
    let config = RuntimeConfig::default();
    update_global_runtime_config(config);
    info!("ランタイム設定をリセットしました");
    Ok(())
}

/// ランタイム設定値を設定する
///
/// # 引数
///
/// * `key` - 設定キー
/// * `value` - 設定値
///
/// # 戻り値
///
/// 設定の更新に成功した場合は`Ok(())`、失敗した場合は`Err`
pub fn set_runtime_value(key: &str, value: &str) -> Result<(), BallistaError> {
    trace_fn!("set_runtime_value(key: {}, value: {})", key, value);
    
    if let Ok(mut runtime_config) = RUNTIME_CONFIG.lock() {
        if let Some(config) = &mut *runtime_config {
            config.values.insert(key.to_string(), value.to_string());
            debug!("ランタイム設定値を設定しました: {} = {}", key, value);
            return Ok(());
        }
    }
    
    Err(BallistaError::ConfigError {
        message: "ランタイム設定が初期化されていません".to_string(),
        context_info: String::new(),
    })
}

/// ランタイム設定値を取得する
///
/// # 引数
///
/// * `key` - 設定キー
///
/// # 戻り値
///
/// 設定値。設定が存在しない場合は`None`
pub fn get_runtime_value(key: &str) -> Option<String> {
    trace_fn!("get_runtime_value(key: {})", key);
    
    if let Ok(runtime_config) = RUNTIME_CONFIG.lock() {
        if let Some(config) = &*runtime_config {
            return config.values.get(key).cloned();
        }
    }
    
    None
}

/// ランタイム設定値を削除する
///
/// # 引数
///
/// * `key` - 設定キー
///
/// # 戻り値
///
/// 設定の削除に成功した場合は`Ok(())`、失敗した場合は`Err`
pub fn remove_runtime_value(key: &str) -> Result<(), BallistaError> {
    trace_fn!("remove_runtime_value(key: {})", key);
    
    if let Ok(mut runtime_config) = RUNTIME_CONFIG.lock() {
        if let Some(config) = &mut *runtime_config {
            config.values.remove(key);
            debug!("ランタイム設定値を削除しました: {}", key);
            return Ok(());
        }
    }
    
    Err(BallistaError::ConfigError {
        message: "ランタイム設定が初期化されていません".to_string(),
        context_info: String::new(),
    })
}

/// ランタイム設定値が存在するかどうかを確認する
///
/// # 引数
///
/// * `key` - 設定キー
///
/// # 戻り値
///
/// 設定値が存在する場合は`true`、存在しない場合は`false`
pub fn has_runtime_value(key: &str) -> bool {
    trace_fn!("has_runtime_value(key: {})", key);
    
    if let Ok(runtime_config) = RUNTIME_CONFIG.lock() {
        if let Some(config) = &*runtime_config {
            return config.values.contains_key(key);
        }
    }
    
    false
}

/// 現在のランタイム設定をログに出力する
pub fn log_runtime_config() {
    trace_fn!("log_runtime_config()");
    if let Some(config) = get_runtime_config() {
        debug!("ランタイム設定: セッションID = {}", config.session_id);
        debug!("ランタイム設定値:");
        for (key, value) in &config.values {
            debug!("  {} = {}", key, value);
        }
    } else {
        debug!("ランタイム設定が初期化されていません");
    }
}

/// 便利なアクセス関数: modding.xmlのパスを設定する
///
/// # 引数
///
/// * `path` - modding.xmlのパス
///
/// # 戻り値
///
/// 設定の更新に成功した場合は`Ok(())`、失敗した場合は`Err`
pub fn set_modding_xml_path(path: &str) -> Result<(), BallistaError> {
    trace_fn!("set_modding_xml_path(path: {})", path);
    set_runtime_value("modding_xml_path", path)
}

/// 便利なアクセス関数: modding.xmlのパスを取得する
///
/// # 戻り値
///
/// modding.xmlのパス。設定が存在しない場合は`None`
pub fn get_modding_xml_path() -> Option<String> {
    trace_fn!("get_modding_xml_path()");
    get_runtime_value("modding_xml_path")
}

/// ランタイム設定が初期化されているかどうかを確認する
fn check_runtime_config_initialized() -> Result<(), BallistaError> {
    trace_fn!("check_runtime_config_initialized()");
    
    if let Ok(runtime_config) = RUNTIME_CONFIG.lock() {
        if runtime_config.is_some() {
            return Ok(());
        }
    }
    
    Err(BallistaError::ConfigError {
        message: "ランタイム設定が初期化されていません".to_string(),
        context_info: String::new(),
    })
}

/// ModdingXMLのパスが設定されているかどうかを確認する
fn check_modding_xml_path_set() -> Result<(), BallistaError> {
    trace_fn!("check_modding_xml_path_set()");
    
    if let Ok(runtime_config) = RUNTIME_CONFIG.lock() {
        if let Some(config) = &*runtime_config {
            if config.values.contains_key("modding_xml_path") {
                return Ok(());
            }
        }
    }
    
    Err(BallistaError::ConfigError {
        message: "ランタイム設定が初期化されていません".to_string(),
        context_info: String::new(),
    })
}
