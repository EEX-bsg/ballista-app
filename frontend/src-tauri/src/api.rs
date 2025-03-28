/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tauri APIコマンドモジュール
//! このモジュールはTauriのコマンドを定義し、ビジネスロジックと分離します

// ライブラリクレートからのインポート
use ballista_app::{
    core::modding::{self, ModInfo, ModdingXmlData},
    core::backup::{self, BackupInfo},
    core::preset::{self, PresetInfo, PresetData},
    config::{self, load_config, update_config, get_log_level, AppConfig},
    error::BallistaError,
    logger::set_log_level,
    trace_fn,
};

use anyhow::Result;
use log::{info, warn};
use std::path::PathBuf;
use tauri::api::dialog::blocking::FileDialogBuilder;
use tauri::AppHandle;

pub mod file_operations {
    //! ファイル操作系API
    use super::*;

    /// ファイル選択ダイアログを表示し、Modding.xmlを選択する
    #[tauri::command]
    pub async fn select_modding_xml() -> Result<String, String> {
        trace_fn!("select_modding_xml()");
        info!("ファイル選択ダイアログを表示します");
        
        let file_path: PathBuf = FileDialogBuilder::new()
            .add_filter("XML", &["xml"])
            .pick_file()
            .ok_or_else(|| "ファイルが選択されませんでした".to_string())?;
        
        info!("ファイルが選択されました: {}", file_path.display());
        
        // ファイルを読み込む
        match modding::read_modding_xml(&file_path) {
            Ok(_content) => Ok(file_path.to_string_lossy().to_string()),
            Err(e) => Err(e),
        }
    }

    /// 指定されたパスのModding.xmlファイルを読み込む
    #[tauri::command]
    pub async fn read_modding_xml(file_path: String) -> Result<ModdingXmlData, String> {
        trace_fn!("read_modding_xml(file_path: {})", file_path);
        
        let path = PathBuf::from(file_path);
        
        // ファイルを読み込む
        let content = modding::read_modding_xml(&path)?;
        
        // XMLを解析
        modding::parse_modding_xml(&content, Some(path))
    }

    /// 指定されたパスにModding.xmlファイルを書き込む
    #[tauri::command]
    pub async fn write_modding_xml(file_path: String, data: ModdingXmlData) -> Result<(), String> {
        trace_fn!("write_modding_xml(file_path: {})", file_path);
        
        let path = PathBuf::from(file_path);
        
        // バックアップを作成
        match backup::create_backup(&path) {
            Ok(_) => (),
            Err(e) => warn!("バックアップの作成に失敗しました: {}", e),
        }
        
        // ファイルを書き込む
        modding::write_modding_xml(&path, &data)
    }

    /// パスの有効性を検証する
    #[tauri::command]
    pub async fn validate_path(path: String) -> Result<bool, String> {
        trace_fn!("validate_path(path: {})", path);
        
        let path_obj = PathBuf::from(path);
        
        match modding::validate_path(&path_obj) {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }
}

pub mod xml_operations {
    //! XML操作系API
    use super::*;

    /// モッドの有効/無効を切り替える
    #[tauri::command]
    pub async fn toggle_mod_enabled(data: ModdingXmlData, uuid: String) -> Result<ModdingXmlData, String> {
        trace_fn!("toggle_mod_enabled(uuid: {})", uuid);
        
        let mut data_copy = data;
        modding::toggle_mod_enabled(&mut data_copy, &uuid)?;
        
        Ok(data_copy)
    }

    /// モッド情報を取得する
    #[tauri::command]
    pub async fn get_mod_info(data: ModdingXmlData, uuid: String) -> Result<Option<ModInfo>, String> {
        trace_fn!("get_mod_info(uuid: {})", uuid);
        
        Ok(modding::get_mod_info(&data, &uuid))
    }

    /// モッドを有効化する
    #[tauri::command]
    pub async fn enable_mod(uuid: String) -> Result<(), String> {
        trace_fn!("enable_mod(uuid: {})", uuid);
        
        // ファイルを選択
        let file_path = file_operations::select_modding_xml().await?;
        
        // ファイルを読み込む
        let mut data = file_operations::read_modding_xml(file_path.clone()).await?;
        
        // モッドを有効化
        if let Some(pos) = data.disabled_mods.iter().position(|m| m.uuid == uuid) {
            let mut mod_info = data.disabled_mods.remove(pos);
            mod_info.enabled = true;
            data.enabled_mods.push(mod_info);
        }
        
        // ファイルを書き込む
        file_operations::write_modding_xml(file_path, data).await
    }

    /// モッドを無効化する
    #[tauri::command]
    pub async fn disable_mod(uuid: String) -> Result<(), String> {
        trace_fn!("disable_mod(uuid: {})", uuid);
        
        // ファイルを選択
        let file_path = file_operations::select_modding_xml().await?;
        
        // ファイルを読み込む
        let mut data = file_operations::read_modding_xml(file_path.clone()).await?;
        
        // モッドを無効化
        if let Some(pos) = data.enabled_mods.iter().position(|m| m.uuid == uuid) {
            let mut mod_info = data.enabled_mods.remove(pos);
            mod_info.enabled = false;
            data.disabled_mods.push(mod_info);
        }
        
        // ファイルを書き込む
        file_operations::write_modding_xml(file_path, data).await
    }
}

pub mod backup_operations {
    //! バックアップ系API
    use super::*;

    /// バックアップファイルを作成する
    #[tauri::command]
    pub async fn create_backup_file(file_path: String) -> Result<String, String> {
        trace_fn!("create_backup_file(file_path: {})", file_path);
        
        let path = PathBuf::from(file_path);
        
        match backup::create_backup(&path) {
            Ok(backup_path) => Ok(backup_path.to_string_lossy().to_string()),
            Err(e) => Err(e),
        }
    }

    /// バックアップ一覧を取得する
    #[tauri::command]
    pub async fn get_backup_list(file_path: String) -> Result<Vec<BackupInfo>, String> {
        trace_fn!("get_backup_list(file_path: {})", file_path);
        
        let path = PathBuf::from(file_path);
        
        backup::get_backup_list(&path)
    }

    /// バックアップを復元する
    #[tauri::command]
    pub async fn restore_backup(backup_path: String, target_path: String) -> Result<(), String> {
        trace_fn!("restore_backup(backup_path: {}, target_path: {})", backup_path, target_path);
        
        let backup = PathBuf::from(backup_path);
        let target = PathBuf::from(target_path);
        
        backup::restore_backup(&backup, &target)
    }
}

pub mod preset_operations {
    //! プリセット系API
    use super::*;

    /// プリセットを保存する
    #[tauri::command]
    pub async fn save_preset(name: String, description: Option<String>, data: ModdingXmlData) -> Result<String, String> {
        trace_fn!("save_preset(name: {}, description: {:?})", name, description);
        
        match preset::save_preset(&name, description.as_deref(), &data) {
            Ok(path) => Ok(path.to_string_lossy().to_string()),
            Err(e) => Err(e),
        }
    }

    /// プリセット一覧を取得する
    #[tauri::command]
    pub async fn get_preset_list() -> Result<Vec<PresetInfo>, String> {
        trace_fn!("get_preset_list()");
        
        preset::get_preset_list()
    }

    /// プリセットを読み込む
    #[tauri::command]
    pub async fn load_preset(preset_path: String) -> Result<PresetData, String> {
        trace_fn!("load_preset(preset_path: {})", preset_path);
        
        let path = PathBuf::from(preset_path);
        
        preset::load_preset(&path)
    }

    /// プリセットを適用する
    #[tauri::command]
    pub async fn apply_preset(preset_path: String, target_path: String) -> Result<(), String> {
        trace_fn!("apply_preset(preset_path: {}, target_path: {})", preset_path, target_path);
        
        let preset = PathBuf::from(preset_path);
        let target = PathBuf::from(target_path);
        
        preset::apply_preset(&preset, &target)
    }

    /// プリセットを削除する
    #[tauri::command]
    pub async fn delete_preset(preset_path: String) -> Result<(), String> {
        trace_fn!("delete_preset(preset_path: {})", preset_path);
        
        let path = PathBuf::from(preset_path);
        
        preset::delete_preset(&path)
    }

    /// プリセット一覧を取得する簡易バージョン
    #[tauri::command]
    pub async fn load_presets() -> Result<Vec<PresetInfo>, String> {
        trace_fn!("load_presets()");
        
        // プリセット一覧を取得
        get_preset_list().await
    }
}

pub mod app_operations {
    //! アプリケーション全般の操作API
    use super::*;

    /// アプリケーション設定を取得する
    #[tauri::command]
    pub async fn get_config(app_handle: AppHandle) -> Result<AppConfig, String> {
        trace_fn!("get_config()");
        Ok(load_config(&app_handle))
    }

    /// ログレベルを設定する
    #[tauri::command]
    pub async fn set_log_level_command(app_handle: AppHandle, level: String) -> Result<(), String> {
        trace_fn!("set_log_level_command(level: {})", level);
        // 設定ファイルからコンフィグを読み込む
        let mut config = load_config(&app_handle);
        
        // ログレベルを更新
        config.logging.level = level.clone();
        
        // 設定を保存
        config::save_config(&app_handle, &config);
        
        // 設定を反映
        update_config(&app_handle);
        
        // ロガーのレベルを設定
        set_log_level(get_log_level(&level));
        
        info!("ログレベルを変更しました: {}", level);
        
        Ok(())
    }

    /// ログメッセージを記録する
    /// Vue側からのログを保存する用
    #[tauri::command]
    pub fn log_message(level: String, message: String) {
        // 二重トレースになっちゃうから無効化
        // trace_fn!("log_message(level: {}, message: {})", level, message);
        match level.to_lowercase().as_str() {
            "trace" => log::trace!("{}", message),
            "debug" => log::debug!("{}", message),
            "info" => log::info!("{}", message),
            "warn" => log::warn!("{}", message),
            "error" => log::error!("{}", message),
            _ => log::info!("{}", message),
        }
    }

    /// ログを取得する
    #[tauri::command]
    pub async fn get_logs(count: Option<usize>) -> Vec<String> {
        trace_fn!("get_logs(count: {:?})", count);
        // ここでログを取得する実装
        // 実際のログはtauri-plugin-logが保存している
        vec!["ログの実装は別途行います".to_string()]
    }

    /// エラーハンドリングAPI
    #[tauri::command]
    pub fn handle_error(error_msg: String) -> Result<Option<String>, String> {
        trace_fn!("handle_error(error_msg: {})", error_msg);
        
        let error: BallistaError = error_msg.into();
        error.log();
        error.try_recover()
    }
}

pub mod modding_integration {
    //! モッド統合系API
    use super::*;

    /// Modding.xmlを読み込む統合API
    #[tauri::command]
    pub async fn load_modding_xml() -> Result<ModdingXmlData, String> {
        trace_fn!("load_modding_xml()");
        
        // ファイルを選択
        let file_path = file_operations::select_modding_xml().await?;
        
        // ファイルを読み込む
        file_operations::read_modding_xml(file_path).await
    }
}