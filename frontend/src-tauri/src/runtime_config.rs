/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! ランタイム設定管理モジュール
//!
//! このモジュールは、アプリケーション実行中のみ保持するグローバル設定を管理するための機能を提供します。
//! これらの設定はメモリ上にのみ存在し、アプリケーションが終了すると消えます。

use crate::config;
use crate::constants::*;
use crate::error::BallistaError;
use crate::trace_fn;
use crate::utils::normalize_path;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// ランタイム設定のキー定数
pub const RUNTIME_KEY_MODDING_XML_PATH: &str = "modding_xml_path";
pub const RUNTIME_KEY_MODS_DIR_PATH: &str = "mods_dir_path";
pub const RUNTIME_KEY_BESIEGE_WORKSHOP_PATH: &str = "besiege_workshop_path";

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
    info!(
        "ランタイム設定を初期化しました: セッションID = {}",
        config.session_id
    );

    // 派生パスを初期化
    initialize_derived_paths();
}

/// 派生パスを初期化する
/// Besiege.exeパスとworkshopパスから派生するパスを設定
fn initialize_derived_paths() {
    trace_fn!("initialize_derived_paths()");

    // Besiege.exeパスから派生するパスを設定
    update_paths_from_besiege_exe();

    // Workshopパスから派生するパスを設定
    update_paths_from_workshop();
}

/// パスをPathBufから正規化された文字列に変換する
fn path_to_normalized_string(path: &Path) -> String {
    normalize_path(&path.to_string_lossy().to_string())
}

/// Besiege.exeパスから派生するパスを更新
pub fn update_paths_from_besiege_exe() {
    trace_fn!("update_paths_from_besiege_exe()");

    let besiege_path = config::get_besiege_path();
    if besiege_path.is_empty() {
        debug!("Besiege.exeパスが設定されていないため、派生パスを初期化できません");
        return;
    }

    let besiege_dir = Path::new(&besiege_path).parent();
    if let Some(dir) = besiege_dir {
        // modding.xmlパスを設定
        let modding_xml_path = dir.join(MODDING_CONFIG_RELATIVE_PATH);
        let modding_xml_path_str = path_to_normalized_string(&modding_xml_path);
        let _ = set_runtime_value(RUNTIME_KEY_MODDING_XML_PATH, &modding_xml_path_str);

        // modsディレクトリパスを設定
        let mods_dir_path = dir.join(MODS_DIR_RELATIVE_PATH);
        let mods_dir_path_str = path_to_normalized_string(&mods_dir_path);
        let _ = set_runtime_value(RUNTIME_KEY_MODS_DIR_PATH, &mods_dir_path_str);

        debug!(
            "Besiege.exeから派生パスを更新しました: modding.xml: {}, mods: {}",
            modding_xml_path_str, mods_dir_path_str
        );
    } else {
        debug!("Besiege.exeの親ディレクトリを取得できないため、派生パスを初期化できません");
    }
}

/// Workshopパスから派生するパスを更新
pub fn update_paths_from_workshop() {
    trace_fn!("update_paths_from_workshop()");

    let workshop_path = config::get_workshop_path();
    if workshop_path.is_empty() {
        debug!("Workshopパスが設定されていないため、派生パスを初期化できません");
        return;
    }

    // BesiegeWorkshopパスを設定
    let workshop_dir = Path::new(&workshop_path);
    let besiege_workshop_path = workshop_dir.join(BESIEGE_WORKSHOP_DIR_RELATIVE_PATH);
    let besiege_workshop_path_str = path_to_normalized_string(&besiege_workshop_path);
    let _ = set_runtime_value(
        RUNTIME_KEY_BESIEGE_WORKSHOP_PATH,
        &besiege_workshop_path_str,
    );

    debug!(
        "Workshopから派生パスを更新しました: BesiegeWorkshop: {}",
        besiege_workshop_path_str
    );
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

    // 派生パスを再初期化
    initialize_derived_paths();

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

/// modding.xmlのパスを取得する
pub fn get_modding_xml_path() -> Option<String> {
    trace_fn!("get_modding_xml_path()");
    get_runtime_value(RUNTIME_KEY_MODDING_XML_PATH)
}

/// modsディレクトリのパスを取得する
pub fn get_mods_dir_path() -> Option<String> {
    trace_fn!("get_mods_dir_path()");
    get_runtime_value(RUNTIME_KEY_MODS_DIR_PATH)
}

/// BesiegeWorkshopディレクトリのパスを取得する
pub fn get_besiege_workshop_path() -> Option<String> {
    trace_fn!("get_besiege_workshop_path()");
    get_runtime_value(RUNTIME_KEY_BESIEGE_WORKSHOP_PATH)
}
