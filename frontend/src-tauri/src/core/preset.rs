/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Local};
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use crate::core::modding::{ModInfo, ModdingXmlData};
use crate::core::backup;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
    pub mod_count: usize,
    pub enabled_count: usize,
    pub disabled_count: usize,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetData {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
    pub mods: Vec<ModInfo>,
    pub game_version: Option<String>,
}

/// プリセットディレクトリを取得する
pub fn get_preset_dir() -> Result<PathBuf, String> {
    trace_fn!("get_preset_dir()");
    
    // アプリケーションデータディレクトリを取得
    let data_dir = dirs_next::data_dir()
        .ok_or_else(|| "アプリケーションデータディレクトリが見つかりません".to_string())?;
    
    // プリセットディレクトリのパスを生成
    let preset_dir = data_dir.join("ballista-app").join("presets");
    
    // ディレクトリが存在しない場合は作成
    if !preset_dir.exists() {
        if let Err(e) = fs::create_dir_all(&preset_dir) {
            return Err(format!("プリセットディレクトリの作成に失敗しました: {}", e));
        }
    }
    
    debug!("プリセットディレクトリ: {}", preset_dir.display());
    Ok(preset_dir)
}

/// プリセットを保存する
pub fn save_preset(name: &str, description: Option<&str>, data: &ModdingXmlData) -> Result<PathBuf, String> {
    trace_fn!("save_preset(name: {}, description: {:?})", name, description);
    
    // プリセット名のバリデーション
    if name.trim().is_empty() {
        return Err("プリセット名は空にできません".to_string());
    }
    
    if name.contains(|c: char| c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|') {
        return Err("プリセット名に無効な文字が含まれています".to_string());
    }
    
    // プリセットディレクトリを取得
    let preset_dir = get_preset_dir()?;
    
    // プリセットファイルのパスを生成
    let file_name = format!("{}.json", name);
    let preset_path = preset_dir.join(&file_name);
    
    // プリセットデータを作成
    let current_time = Local::now();
    let preset_data = PresetData {
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        created_at: current_time,
        modified_at: current_time,
        mods: data.enabled_mods.iter().chain(data.disabled_mods.iter()).cloned().collect(),
        game_version: data.game_version.clone(),
    };
    
    // JSONに変換
    let json = match serde_json::to_string_pretty(&preset_data) {
        Ok(json) => json,
        Err(e) => return Err(format!("プリセットデータのJSON変換に失敗しました: {}", e)),
    };
    
    // ファイルに書き込む
    if let Err(e) = fs::write(&preset_path, json) {
        return Err(format!("プリセットファイルの書き込みに失敗しました: {}", e));
    }
    
    info!("プリセットを保存しました: {}", preset_path.display());
    Ok(preset_path)
}

/// プリセットを読み込む
pub fn load_preset(preset_path: &Path) -> Result<PresetData, String> {
    trace_fn!("load_preset(preset_path: {})", preset_path.display());
    
    // ファイルが存在することを確認
    if !preset_path.exists() {
        return Err(format!("プリセットファイルが存在しません: {}", preset_path.display()));
    }
    
    // ファイル内容を読み込み
    let content = match fs::read_to_string(preset_path) {
        Ok(content) => content,
        Err(e) => return Err(format!("プリセットファイルの読み込みに失敗しました: {}", e)),
    };
    
    // JSONをパース
    match serde_json::from_str::<PresetData>(&content) {
        Ok(preset) => {
            debug!("プリセットを読み込みました: {}", preset_path.display());
            Ok(preset)
        },
        Err(e) => Err(format!("プリセットデータのパースに失敗しました: {}", e)),
    }
}

/// プリセット一覧を取得する
pub fn get_preset_list() -> Result<Vec<PresetInfo>, String> {
    trace_fn!("get_preset_list()");
    
    // プリセットディレクトリを取得
    let preset_dir = get_preset_dir()?;
    
    // プリセットファイルのリストを取得
    let mut presets = Vec::new();
    
    for entry in WalkDir::new(&preset_dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            match load_preset(path) {
                Ok(preset_data) => {
                    let enabled_count = preset_data.mods.iter().filter(|m| m.enabled).count();
                    let disabled_count = preset_data.mods.len() - enabled_count;
                    
                    presets.push(PresetInfo {
                        name: preset_data.name,
                        description: preset_data.description,
                        created_at: preset_data.created_at,
                        modified_at: preset_data.modified_at,
                        mod_count: preset_data.mods.len(),
                        enabled_count,
                        disabled_count,
                        path: path.to_path_buf(),
                    });
                },
                Err(e) => {
                    warn!("プリセットの読み込みに失敗しました: {}: {}", path.display(), e);
                }
            }
        }
    }
    
    // 更新日時で新しい順にソート
    presets.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    
    debug!("プリセット一覧を取得しました: {} 件", presets.len());
    Ok(presets)
}

/// プリセットを適用する
pub fn apply_preset(preset_file: &Path, target_file: &Path) -> Result<(), String> {
    trace!("apply_preset(preset_file: {}, target_file: {})", preset_file.display(), target_file.display());
    
    // プリセットを読み込む
    let preset_data = load_preset(preset_file)?;
    
    // プリセットからModdingXmlDataを作成
    let enabled_mods: Vec<ModInfo> = preset_data.mods.iter()
        .filter(|m| m.enabled)
        .cloned()
        .collect();
    
    let disabled_mods: Vec<ModInfo> = preset_data.mods.iter()
        .filter(|m| !m.enabled)
        .cloned()
        .collect();
    
    let modding_data = ModdingXmlData {
        enabled_mods,
        disabled_mods,
        game_version: preset_data.game_version,
        file_path: Some(target_file.to_path_buf()),
    };
    
    // バックアップを作成
    match backup::create_backup(target_file) {
        Ok(_) => (),
        Err(e) => warn!("バックアップの作成に失敗しました: {}", e),
    }
    
    // 対象ファイルに書き込み
    match crate::core::modding::write_modding_xml(target_file, &modding_data) {
        Ok(_) => {
            info!("プリセットを適用しました: {} -> {}", preset_file.display(), target_file.display());
            Ok(())
        },
        Err(e) => Err(format!("プリセットの適用に失敗しました: {}", e)),
    }
}

/// プリセットを削除する
pub fn delete_preset(preset_path: &Path) -> Result<(), String> {
    trace_fn!("delete_preset(preset_path: {})", preset_path.display());
    
    // ファイルが存在することを確認
    if !preset_path.exists() {
        return Err(format!("プリセットファイルが存在しません: {}", preset_path.display()));
    }
    
    // ファイルを削除
    match fs::remove_file(preset_path) {
        Ok(_) => {
            info!("プリセットを削除しました: {}", preset_path.display());
            Ok(())
        },
        Err(e) => Err(format!("プリセットの削除に失敗しました: {}", e)),
    }
} 