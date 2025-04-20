/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tauri APIコマンドモジュール
//! このモジュールはTauriのコマンドを定義し、ビジネスロジックと分離します

// ライブラリクレートからのインポート
use ballista_app::{
    core::modding::{self, ModInfo, ModListData},
    core::preset::{self, PresetInfo, PresetData},
    config::{self, load_config, update_config, get_log_level, AppConfig},
    runtime_config::{self, RuntimeConfig},
    error::BallistaError,
    logger::set_log_level,
    trace_fn,
};

use anyhow::Result;
use log::{LevelFilter, info, warn};
use std::path::PathBuf;
use tauri::api::dialog::blocking::FileDialogBuilder;
use tauri::AppHandle;

pub mod file_operations {
    //! ファイル操作系API
    use super::*;
}

pub mod preset_operations {
    //! プリセット系API
    use super::*;

}

pub mod config_operations {
    //! 設定関連のAPI
    use super::*;

    /// アプリケーション設定を取得する
    #[tauri::command]
    pub async fn get_config(app_handle: AppHandle) -> Result<AppConfig, BallistaError> {
        trace_fn!("get_config()");
        Ok(load_config(&app_handle))
    }

    /// Besiegeのパスを取得する
    #[tauri::command]
    pub async fn get_besiege_path() -> Result<String, BallistaError> {
        trace_fn!("get_besiege_path()");
        Ok(config::get_besiege_path())
    }

    /// Besiegeのパスを設定する
    #[tauri::command]
    pub async fn set_besiege_path(app_handle: AppHandle, path: String) -> Result<(), BallistaError> {
        trace_fn!("set_besiege_path(path: {})", path);
        config::set_besiege_path(&app_handle, &path)
    }

    /// Steam Workshopのパスを取得する
    #[tauri::command]
    pub async fn get_workshop_path() -> Result<String, BallistaError> {
        trace_fn!("get_workshop_path()");
        Ok(config::get_workshop_path())
    }

    /// Steam Workshopのパスを設定する
    #[tauri::command]
    pub async fn set_workshop_path(app_handle: AppHandle, path: String) -> Result<(), BallistaError> {
        trace_fn!("set_workshop_path(path: {})", path);
        config::set_workshop_path(&app_handle, &path)
    }

    /// Ballistaデータパスを取得する
    #[tauri::command]
    pub async fn get_ballista_data_path() -> Result<String, BallistaError> {
        trace_fn!("get_ballista_data_path()");
        Ok(config::get_ballista_data_path())
    }

    /// Ballistaデータパスを設定する
    #[tauri::command]
    pub async fn set_ballista_data_path(app_handle: AppHandle, path: String) -> Result<(), BallistaError> {
        trace_fn!("set_ballista_data_path(path: {})", path);
        config::set_ballista_data_path(&app_handle, &path)
    }

    /// UIテーマを取得する
    #[tauri::command]
    pub async fn get_ui_theme() -> Result<String, BallistaError> {
        trace_fn!("get_ui_theme()");
        Ok(config::get_ui_theme())
    }

    /// UIテーマを設定する
    #[tauri::command]
    pub async fn set_ui_theme(app_handle: AppHandle, theme: String) -> Result<(), BallistaError> {
        trace_fn!("set_ui_theme(theme: {})", theme);
        config::set_ui_theme(&app_handle, &theme)
    }

    /// 言語設定を取得する
    #[tauri::command]
    pub async fn get_language() -> Result<String, BallistaError> {
        trace_fn!("get_language()");
        Ok(config::get_language())
    }

    /// 言語設定を設定する
    #[tauri::command]
    pub async fn set_language(app_handle: AppHandle, language: String) -> Result<(), BallistaError> {
        trace_fn!("set_language(language: {})", language);
        config::set_language(&app_handle, &language)
    }
}

pub mod runtime_config_operations {
    //! ランタイム設定関連のAPI
    use super::*;

    /// ランタイム設定を取得する
    #[tauri::command]
    pub async fn get_runtime_config() -> Result<Option<RuntimeConfig>, BallistaError> {
        trace_fn!("get_runtime_config()");
        Ok(runtime_config::get_runtime_config())
    }

    /// ランタイム設定をリセットする
    #[tauri::command]
    pub async fn reset_runtime_config() -> Result<(), BallistaError> {
        trace_fn!("reset_runtime_config()");
        runtime_config::reset_runtime_config()
    }

    /// ランタイム設定値を設定する
    #[tauri::command]
    pub async fn set_runtime_value(key: String, value: String) -> Result<(), BallistaError> {
        trace_fn!("set_runtime_value(key: {}, value: {})", key, value);
        runtime_config::set_runtime_value(&key, &value)
    }

    /// ランタイム設定値を取得する
    #[tauri::command]
    pub async fn get_runtime_value(key: String) -> Result<Option<String>, BallistaError> {
        trace_fn!("get_runtime_value(key: {})", key);
        Ok(runtime_config::get_runtime_value(&key))
    }

    /// ランタイム設定値を削除する
    #[tauri::command]
    pub async fn remove_runtime_value(key: String) -> Result<(), BallistaError> {
        trace_fn!("remove_runtime_value(key: {})", key);
        runtime_config::remove_runtime_value(&key)
    }

    /// ランタイム設定値が存在するかどうかを確認する
    #[tauri::command]
    pub async fn has_runtime_value(key: String) -> Result<bool, BallistaError> {
        trace_fn!("has_runtime_value(key: {})", key);
        Ok(runtime_config::has_runtime_value(&key))
    }
}

pub mod app_operations {
    //! アプリケーション全般の操作API
    use super::*;

    /// ログメッセージを記録する
    /// Vue側からのログを保存する用
    #[tauri::command]
    pub fn log_message(level: String, message: String) {
        match level.to_lowercase().as_str() {
            "trace" => log::trace!("{}", message),
            "debug" => log::debug!("{}", message),
            "info" => log::info!("{}", message),
            "warn" => log::warn!("{}", message),
            "error" => log::error!("{}", message),
            _ => log::info!("{}", message),
        }
    }

    /// エラーハンドリングAPI
    #[tauri::command]
    pub fn handle_error(error: BallistaError) -> Result<String, BallistaError> {
        error.log();
        Ok(error.user_message())
    }
}
