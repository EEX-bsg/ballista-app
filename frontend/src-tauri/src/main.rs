/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;

// クレートをライブラリとしても使用できるようにする
use ballista_app::{
    logger::{self, set_log_level},
    config::{load_config, get_log_level, update_config, APP_CONFIG},
    watcher::{start_config_watcher, handle_config_changes},
};

use anyhow::Result;
use chrono::Local;
use env_logger::Builder;
use log::{error, info, LevelFilter};
use std::fs;
use std::path::PathBuf;
use std::thread;
use tauri::{AppHandle, Manager};

// ログレベルを取得する関数
fn get_current_log_level() -> LevelFilter {
    // 設定からログレベルを取得
    get_log_level()
}

// 更新可能なロガーを設定
fn setup_logger(level: LevelFilter) {
    let mut builder = Builder::new();
    builder.filter_level(level);
    builder.init();
    info!("ロガーを設定しました: レベル = {:?}", level);
}

// Tauriの初期化関数
fn init_app(app: &AppHandle) {
    // 設定ファイルの読み込み
    update_config(app);
    
    // ファイル監視の開始
    match start_config_watcher(app.clone()) {
        Ok(rx) => {
            let app_handle = app.clone();
            thread::spawn(move || {
                for res in rx {
                    match res {
                        Ok(event) => {
                            handle_config_changes(event, &app_handle);
                        }
                        Err(e) => error!("ファイル監視エラー: {}", e),
                    }
                }
            });
        }
        Err(e) => error!("ファイル監視の開始に失敗しました: {}", e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログファイルのパスを設定
    let log_dir = dirs_next::data_dir()
        .map(|p| p.join("ballista-app/logs"))
        .unwrap_or_else(|| PathBuf::from("logs"));
    
    // ログディレクトリを作成
    fs::create_dir_all(&log_dir)?;
    
    // ログファイルのパス
    let log_file = log_dir.join(format!("ballista_{}.log", 
        Local::now().format("%Y%m%d_%H%M%S")));
    
    // ロガーを初期化
    logger::init_logger(Some(log_file))?;
    
    info!("アプリケーションを起動しています");
    
    tauri::Builder::default()
        .setup(|app| {
            // アプリケーションの初期化
            init_app(&app.app_handle());
            
            // ログレベルを設定ファイルから取得して更新
            let level = get_current_log_level();
            set_log_level(level);
            info!("ログレベルを設定しました: {:?}", level);
            
            Ok(())
        })
        .on_page_load(|window, _| {
            // 必要に応じてフロントエンドにイベントを送信
            let app_handle = window.app_handle();
            let config = load_config(&app_handle);
            window.emit("config:loaded", config).unwrap();
        })
        .invoke_handler(tauri::generate_handler![
            // ファイル操作系API
            api::file_operations::select_modding_xml,
            api::file_operations::read_modding_xml,
            // api::file_operations::write_modding_xml,
            api::file_operations::validate_path,
            
            // XML操作系API
            // api::xml_operations::toggle_mod_enabled,
            // api::xml_operations::get_mod_info,
            // api::xml_operations::enable_mod,
            // api::xml_operations::disable_mod,
            
            // プリセット系API
            // api::preset_operations::save_preset,
            // api::preset_operations::get_preset_list,
            // api::preset_operations::load_preset,
            // api::preset_operations::load_presets,
            // api::preset_operations::apply_preset,
            // api::preset_operations::delete_preset,
            
            // 設定関連API
            api::config_operations::get_config,
            api::config_operations::get_log_level,
            api::config_operations::set_log_level,
            api::config_operations::get_besiege_path,
            api::config_operations::set_besiege_path,
            api::config_operations::get_workshop_path,
            api::config_operations::set_workshop_path,
            api::config_operations::get_ballista_data_path,
            api::config_operations::set_ballista_data_path,
            api::config_operations::get_ui_theme,
            api::config_operations::set_ui_theme,
            api::config_operations::get_language,
            api::config_operations::set_language,
            
            // アプリケーション操作系API
            api::app_operations::log_message,
            api::app_operations::handle_error,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
        
    info!("アプリケーションを終了しています");
    Ok(())
}
