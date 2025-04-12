/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tauri APIコマンドモジュール
//! このモジュールはTauriのコマンドを定義し、ビジネスロジックと分離します

// ライブラリクレートからのインポート
use ballista_app::{
    core::modding::{self, ModInfo, ModdingXmlData},
    core::preset::{self, PresetInfo, PresetData},
    config::{self, load_config, update_config, get_log_level, AppConfig},
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

    /// ファイル選択ダイアログを表示し、Modding.xmlを選択する
    #[tauri::command]
    pub async fn select_modding_xml() -> Result<String, BallistaError> {
        trace_fn!("select_modding_xml()");
        info!("ファイル選択ダイアログを表示します");
        
        let file_path: PathBuf = FileDialogBuilder::new()
            .add_filter("XML", &["xml"])
            .pick_file()
            .ok_or_else(|| BallistaError::FileError("ファイルが選択されませんでした".to_string()))?;
        
        info!("ファイルが選択されました: {}", file_path.display());
        
        // ファイルを読み込む（存在確認のため）
        modding::read_modding_xml(&file_path)?;
        
        Ok(file_path.to_string_lossy().to_string())
    }

    /// 指定されたパスのModding.xmlファイルを読み込む
    #[tauri::command]
    pub async fn read_modding_xml(file_path: String) -> Result<ModdingXmlData, BallistaError> {
        trace_fn!("read_modding_xml(file_path: {})", file_path);
        
        let path = PathBuf::from(file_path);
        
        // ファイルを読み込む
        let content = modding::read_modding_xml(&path)?;
        
        // XMLを解析
        modding::parse_modding_xml(&content)
    }

    // /// 指定されたパスにModding.xmlファイルを書き込む
    // #[tauri::command]
    // pub async fn write_modding_xml(file_path: String, data: ModdingXmlData) -> Result<(), String> {
    //     trace_fn!("write_modding_xml(file_path: {})", file_path);
        
    //     let path = PathBuf::from(file_path);
        
    //     // バックアップを作成
    //     match backup::create_backup(&path) {
    //         Ok(_) => (),
    //         Err(e) => warn!("バックアップの作成に失敗しました: {}", e),
    //     }
        
    //     // ファイルを書き込む
    //     modding::write_modding_xml(&path, &data)
    // }

    // /// パスの有効性を検証する
    // #[tauri::command]
    // pub async fn validate_path(path: String) -> Result<bool, String> {
    //     trace_fn!("validate_path(path: {})", path);
        
    //     let path_obj = PathBuf::from(path);
        
    //     match modding::validate_path(&path_obj) {
    //         Ok(_) => Ok(true),
    //         Err(e) => Err(e),
    //     }
    // }
}

pub mod preset_operations {
    //! プリセット系API
    use super::*;

    /// プリセットを保存する
    #[tauri::command]
    pub async fn save_preset(name: String, description: Option<String>, data: ModdingXmlData) -> Result<String, BallistaError> {
        trace_fn!("save_preset(name: {}, description: {:?})", name, description);
        
        let path = preset::save_preset(&name, description.as_deref(), &data)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// プリセット一覧を取得する
    #[tauri::command]
    pub async fn get_preset_list() -> Result<Vec<PresetInfo>, BallistaError> {
        trace_fn!("get_preset_list()");
        preset::get_preset_list()
    }

    /// プリセットを読み込む
    #[tauri::command]
    pub async fn load_preset(preset_path: String) -> Result<PresetData, BallistaError> {
        trace_fn!("load_preset(preset_path: {})", preset_path);
        
        let path = PathBuf::from(preset_path);
        preset::load_preset(&path)
    }

    /// プリセットを適用する
    // #[tauri::command]
    // pub async fn apply_preset(preset_path: String, target_path: String) -> Result<(), BallistaError> {
    //     trace_fn!("apply_preset(preset_path: {}, target_path: {})", preset_path, target_path);
        
    //     let preset = PathBuf::from(preset_path);
    //     let target = PathBuf::from(target_path);
    //     preset::apply_preset(&preset, &target)
    // }

    /// プリセットを削除する
    #[tauri::command]
    pub async fn delete_preset(preset_path: String) -> Result<(), BallistaError> {
        trace_fn!("delete_preset(preset_path: {})", preset_path);
        
        let path = PathBuf::from(preset_path);
        preset::delete_preset(&path)
    }

    /// プリセット一覧を取得する簡易バージョン
    #[tauri::command]
    pub async fn load_presets() -> Result<Vec<PresetInfo>, BallistaError> {
        trace_fn!("load_presets()");
        
        // プリセット一覧を取得
        get_preset_list().await
    }
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

    /// ログレベルを取得する
    // #[tauri::command]
    // pub async fn get_log_level() -> Result<String, BallistaError> {
    //     trace_fn!("get_log_level()");
    //     // LevelFilterを文字列に変換して返す
    //     let level = config::get_log_level();
    //     let level_str = match level {
    //         LevelFilter::Off => "off",
    //         LevelFilter::Error => "error",
    //         LevelFilter::Warn => "warn",
    //         LevelFilter::Info => "info",
    //         LevelFilter::Debug => "debug",
    //         LevelFilter::Trace => "trace",
    //     };
    //     Ok(level_str.to_string())
    // }

    /// ログレベルを設定する
    // #[tauri::command]
    // pub async fn set_log_level(app_handle: AppHandle, level: String) -> Result<(), BallistaError> {
    //     trace_fn!("set_log_level(level: {})", level);
    //     config::set_log_level(&app_handle, &level)
    // }

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
