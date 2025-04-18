/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! プリセット管理モジュール
//!
//! このモジュールは、MOD構成のプリセットを管理するための機能を提供します。

use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use crate::trace_fn;
use crate::error::BallistaError;
use crate::core::modding::ModdingXmlData;

/// プリセット情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
    pub path: String,
}

/// プリセットデータを表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetData {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
    pub mods: ModdingXmlData,
}

/// プリセットディレクトリを取得する
pub fn get_preset_dir() -> Result<PathBuf, BallistaError> {
    trace_fn!("get_preset_dir()");
    let app_data_dir = dirs_next::data_dir()
        .ok_or_else(|| BallistaError::PresetError {
            message: "アプリケーションデータディレクトリが見つかりません".to_string(),
            context_info: String::new(),
        })?;
    
    let preset_dir = app_data_dir.join("ballista-app").join("presets");
    
    // ディレクトリが存在しなければ作成
    if !preset_dir.exists() {
        match fs::create_dir_all(&preset_dir) {
            Ok(_) => debug!("プリセットディレクトリを作成しました: {}", preset_dir.display()),
            Err(e) => return Err(BallistaError::PresetError {
                message: format!("プリセットディレクトリの作成に失敗しました: {}", e),
                context_info: String::new(),
            }),
        }
    }
    
    Ok(preset_dir)
}

/// プリセットを保存する
pub fn save_preset(name: &str, description: Option<&str>, data: &ModdingXmlData) -> Result<PathBuf, BallistaError> {
    trace_fn!("save_preset(name: {}, description: {:?})", name, description);
    
    // プリセット名のバリデーション
    if name.is_empty() {
        return Err(BallistaError::PresetError {
            message: "プリセット名は空にできません".to_string(),
            context_info: String::new(),
        });
    }
    
    if name.contains(|c: char| !c.is_alphanumeric() && c != ' ' && c != '_' && c != '-') {
        return Err(BallistaError::PresetError {
            message: "プリセット名に無効な文字が含まれています".to_string(),
            context_info: String::new(),
        });
    }
    
    // プリセットディレクトリの取得
    let preset_dir = get_preset_dir()?;
    
    // ファイル名の作成（スペースは_に置換）
    let file_name = format!("{}.json", name.replace(' ', "_"));
    let preset_path = preset_dir.join(file_name);
    
    // 現在の時刻を取得
    let now = Local::now();
    
    // プリセットデータの作成
    let preset_data = PresetData {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        created_at: now,
        modified_at: now,
        mods: data.clone(),
    };
    
    // JSONに変換
    let json_data = match serde_json::to_string_pretty(&preset_data) {
        Ok(data) => data,
        Err(e) => return Err(BallistaError::PresetError {
            message: format!("プリセットデータのJSON変換に失敗しました: {}", e),
            context_info: String::new(),
        }),
    };
    
    // ファイルに書き込み
    match fs::write(&preset_path, json_data) {
        Ok(_) => info!("プリセットを保存しました: {}", preset_path.display()),
        Err(e) => return Err(BallistaError::PresetError {
            message: format!("プリセットファイルの書き込みに失敗しました: {}", e),
            context_info: String::new(),
        }),
    }
    
    Ok(preset_path)
}

/// プリセットを読み込む
pub fn load_preset(preset_path: &Path) -> Result<PresetData, BallistaError> {
    trace_fn!("load_preset(preset_path: {})", preset_path.display());
    
    // ファイルの存在確認
    if !preset_path.exists() {
        return Err(BallistaError::PresetError {
            message: format!("プリセットファイルが存在しません: {}", preset_path.display()),
            context_info: String::new(),
        });
    }
    
    // ファイルを読み込む
    let file_content = match fs::read_to_string(preset_path) {
        Ok(content) => content,
        Err(e) => return Err(BallistaError::PresetError {
            message: format!("プリセットファイルの読み込みに失敗しました: {}", e),
            context_info: String::new(),
        }),
    };
    
    // JSONをパース
    match serde_json::from_str::<PresetData>(&file_content) {
        Ok(preset) => {
            debug!("プリセットを読み込みました: {}", preset_path.display());
            Ok(preset)
        },
        Err(e) => Err(BallistaError::PresetError {
            message: format!("プリセットデータのパースに失敗しました: {}", e),
            context_info: String::new(),
        }),
    }
}

/// プリセット一覧を取得する
pub fn get_preset_list() -> Result<Vec<PresetInfo>, BallistaError> {
    trace_fn!("get_preset_list()");
    
    let preset_dir = get_preset_dir()?;
    let mut presets = Vec::new();
    
    if !preset_dir.exists() {
        return Ok(presets);
    }
    
    // ディレクトリを読み込む
    match fs::read_dir(&preset_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    // JSONファイルのみ処理
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                        match load_preset(&path) {
                            Ok(preset_data) => {
                                presets.push(PresetInfo {
                                    name: preset_data.name,
                                    description: preset_data.description,
                                    created_at: preset_data.created_at,
                                    modified_at: preset_data.modified_at,
                                    path: path.to_string_lossy().to_string(),
                                });
                            },
                            Err(e) => {
                                warn!("プリセットの読み込みに失敗しました: {}", e);
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            warn!("プリセットディレクトリの読み込みに失敗しました: {}", e);
        }
    }
    
    // 作成日時の新しい順にソート
    presets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    Ok(presets)
}

/// プリセットを削除する
pub fn delete_preset(preset_path: &Path) -> Result<(), BallistaError> {
    trace_fn!("delete_preset(preset_path: {})", preset_path.display());
    
    // ファイルが存在することを確認
    if !preset_path.exists() {
        return Err(BallistaError::PresetError {
            message: format!("プリセットファイルが存在しません: {}", preset_path.display()),
            context_info: String::new(),
        });
    }
    
    // ファイルを削除
    match fs::remove_file(preset_path) {
        Ok(_) => {
            info!("プリセットを削除しました: {}", preset_path.display());
            Ok(())
        },
        Err(e) => Err(BallistaError::PresetError {
            message: format!("プリセットの削除に失敗しました: {}", e),
            context_info: String::new(),
        }),
    }
}
