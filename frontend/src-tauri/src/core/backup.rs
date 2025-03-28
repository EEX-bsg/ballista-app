/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Local};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub filename: String,
    pub timestamp: DateTime<Local>,
    pub size: u64,
}

/// バックアップファイルを作成する
pub fn create_backup(file_path: &Path) -> Result<PathBuf, String> {
    trace_fn!("create_backup(file_path: {})", file_path.display());
    
    // ファイルが存在することを確認
    if !file_path.exists() {
        return Err(format!("バックアップ対象のファイルが存在しません: {}", file_path.display()));
    }
    
    // タイムスタンプを生成
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    
    // バックアップファイルパスを生成
    let backup_path = if let Some(file_stem) = file_path.file_stem() {
        let mut backup_name = file_stem.to_string_lossy().to_string();
        if let Some(extension) = file_path.extension() {
            backup_name.push_str(".");
            backup_name.push_str(&extension.to_string_lossy());
        }
        backup_name.push_str(".bak.");
        backup_name.push_str(&timestamp);
        
        let parent = file_path.parent().unwrap_or_else(|| Path::new(""));
        parent.join(backup_name)
    } else {
        let mut backup_name = file_path.to_string_lossy().to_string();
        backup_name.push_str(".bak.");
        backup_name.push_str(&timestamp);
        PathBuf::from(backup_name)
    };
    
    // バックアップディレクトリを作成（必要な場合）
    if let Some(parent) = backup_path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(format!("バックアップディレクトリの作成に失敗しました: {}", e));
            }
        }
    }
    
    // ファイルをコピー
    match fs::copy(file_path, &backup_path) {
        Ok(_) => {
            info!("バックアップファイルを作成しました: {}", backup_path.display());
            
            // バックアップファイルの数を管理
            if let Err(e) = cleanup_old_backups(file_path) {
                warn!("古いバックアップの削除中にエラーが発生しました: {}", e);
            }
            
            Ok(backup_path)
        },
        Err(e) => Err(format!("バックアップの作成に失敗しました: {}", e)),
    }
}

/// 古いバックアップファイルを削除する
pub fn cleanup_old_backups(original_file: &Path) -> Result<usize, String> {
    trace_fn!("cleanup_old_backups(original_file: {})", original_file.display());
    
    // 設定からバックアップ保持数を取得
    let backup_count = match config::APP_CONFIG.lock() {
        Ok(config) => {
            if let Some(app_config) = &*config {
                app_config.modding.backup_count
            } else {
                10 // デフォルト値
            }
        },
        Err(_) => 10, // デフォルト値
    };
    
    let parent_dir = original_file.parent().unwrap_or_else(|| Path::new(""));
    let file_prefix = if let Some(stem) = original_file.file_stem() {
        stem.to_string_lossy().to_string()
    } else {
        original_file.to_string_lossy().to_string()
    };
    
    // バックアップファイルのリストを取得
    let mut backups = Vec::new();
    
    for entry in WalkDir::new(parent_dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            if filename.contains(&file_prefix) && filename.contains(".bak.") {
                if let Ok(metadata) = fs::metadata(path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(time) = modified.try_into() {
                            backups.push(BackupInfo {
                                path: path.to_path_buf(),
                                filename: filename.to_string(),
                                timestamp: time,
                                size: metadata.len(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    // タイムスタンプで新しい順にソート
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // 保持数を超える古いバックアップを削除
    let mut deleted_count = 0;
    for backup in backups.iter().skip(backup_count) {
        match fs::remove_file(&backup.path) {
            Ok(_) => {
                debug!("古いバックアップを削除しました: {}", backup.path.display());
                deleted_count += 1;
            },
            Err(e) => {
                warn!("バックアップの削除に失敗しました: {}: {}", backup.path.display(), e);
            },
        }
    }
    
    info!("古いバックアップを {} 件削除しました", deleted_count);
    Ok(deleted_count)
}

/// バックアップファイルのリストを取得する
pub fn get_backup_list(original_file: &Path) -> Result<Vec<BackupInfo>, String> {
    trace_fn!("get_backup_list(original_file: {})", original_file.display());
    
    let parent_dir = original_file.parent().unwrap_or_else(|| Path::new(""));
    let file_prefix = if let Some(stem) = original_file.file_stem() {
        stem.to_string_lossy().to_string()
    } else {
        original_file.to_string_lossy().to_string()
    };
    
    // バックアップファイルのリストを取得
    let mut backups = Vec::new();
    
    for entry in WalkDir::new(parent_dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            if filename.contains(&file_prefix) && filename.contains(".bak.") {
                if let Ok(metadata) = fs::metadata(path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(time) = modified.try_into() {
                            backups.push(BackupInfo {
                                path: path.to_path_buf(),
                                filename: filename.to_string(),
                                timestamp: time,
                                size: metadata.len(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    // タイムスタンプで新しい順にソート
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    debug!("バックアップリストを取得しました: {} 件", backups.len());
    Ok(backups)
}

/// バックアップからファイルを復元する
pub fn restore_backup(backup_path: &Path, target_path: &Path) -> Result<(), String> {
    trace_fn!("restore_backup(backup_path: {}, target_path: {})", 
        backup_path.display(), target_path.display());
    
    // バックアップファイルが存在することを確認
    if !backup_path.exists() {
        return Err(format!("バックアップファイルが存在しません: {}", backup_path.display()));
    }
    
    // 復元前に現在のファイルをバックアップ（復元に失敗した場合のため）
    if target_path.exists() {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let temp_backup = if let Some(file_stem) = target_path.file_stem() {
            let mut backup_name = file_stem.to_string_lossy().to_string();
            if let Some(extension) = target_path.extension() {
                backup_name.push_str(".");
                backup_name.push_str(&extension.to_string_lossy());
            }
            backup_name.push_str(".before_restore.");
            backup_name.push_str(&timestamp);
            
            let parent = target_path.parent().unwrap_or_else(|| Path::new(""));
            parent.join(backup_name)
        } else {
            let mut backup_name = target_path.to_string_lossy().to_string();
            backup_name.push_str(".before_restore.");
            backup_name.push_str(&timestamp);
            PathBuf::from(backup_name)
        };
        
        if let Err(e) = fs::copy(target_path, &temp_backup) {
            return Err(format!("復元前バックアップの作成に失敗しました: {}", e));
        }
        
        debug!("復元前バックアップを作成しました: {}", temp_backup.display());
    }
    
    // バックアップファイルを復元
    match fs::copy(backup_path, target_path) {
        Ok(_) => {
            info!("バックアップから復元しました: {} -> {}", 
                backup_path.display(), target_path.display());
            Ok(())
        },
        Err(e) => Err(format!("バックアップの復元に失敗しました: {}", e)),
    }
} 