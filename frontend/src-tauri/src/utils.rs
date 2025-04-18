/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities
//!
//! 複数のファイルで利用する便利関数などを置く場所

use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use crate::trace_fn;
use crate::error::BallistaError;

/// パスの種類を表すenum
#[derive(Debug, Clone, Copy)]
pub enum PathType {
    File,
    Directory,
}

impl PathType {
    /// エラーメッセージ用の表示文字列を取得
    fn display_name(&self) -> &'static str {
        match self {
            PathType::File => "ファイル",
            PathType::Directory => "ディレクトリ",
        }
    }
}

/// パス文字列を正規化する
///
/// 以下の処理を行います：
/// - 先頭と末尾の空白を削除
/// - バックスラッシュをスラッシュに置換
///
/// # 引数
///
/// * `path` - 正規化するパス文字列
///
/// # 戻り値
///
/// 正規化されたパス文字列
pub fn normalize_path(path: &str) -> String {
    trace_fn!("normalize_path(path: {})", path);
    path.trim().replace('\\', "/").to_string()
}

/// 共通のパス検証を行う内部関数
///
/// 以下の条件をチェックします：
/// - パスが空でないこと
/// - パスが存在すること
/// - パスに読み書き権限があること
///
/// # Arguments
/// * `path` - 検証するパス
/// * `path_type` - パスの種類
fn validate_path(path: &PathBuf, path_type: PathType) -> Result<(), BallistaError> {
    // パスの形式チェック
    if path.to_string_lossy().is_empty() {
        return Err(BallistaError::FileError {
            message: "パスが空です".to_string(),
            context_info: String::new(),
        });
    }
    
    // 存在確認
    if !path.exists() {
        return Err(BallistaError::FileError {
            message: format!("{}が存在しません", path_type.display_name()),
            context_info: format!(": {}", path.display()),
        });
    }
    
    // メタデータによる権限の確認
    match fs::metadata(path) {
        Ok(metadata) => {
            // 読み取り専用かどうかチェック
            if metadata.permissions().readonly() {
                return Err(BallistaError::FileError {
                    message: format!("{}は読み取り専用です", path_type.display_name()),
                    context_info: format!(": {}", path.display()),
                });
            }
        },
        Err(e) => return Err(BallistaError::FileError {
            message: format!("{}のメタデータの取得に失敗しました", path_type.display_name()),
            context_info: format!(": {}", e),
        }),
    }
    
    Ok(())
}

/// ファイルパスを検証する
///
/// 以下の条件をチェックします：
/// - パスが空でないこと
/// - 親ディレクトリが存在すること
/// - ファイルが存在すること（ディレクトリではないこと）
/// - ファイルに読み書き権限があること
///
/// # Arguments
/// * `path` - 検証するファイルパス
pub fn validate_file_path(path: &PathBuf) -> Result<(), BallistaError> {
    trace_fn!("validate_file_path(path: {})", path.display());
    
    // 親ディレクトリの存在確認
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(BallistaError::FileError {
                message: "親ディレクトリが存在しません".to_string(),
                context_info: format!(": {}", parent.display()),
            });
        }
    }
    
    // 共通のパス検証
    validate_path(path, PathType::File)?;
    
    // ファイルタイプの確認
    if !path.is_file() {
        return Err(BallistaError::FileError {
            message: "パスはファイルではありません".to_string(),
            context_info: format!(": {}", path.display()),
        });
    }
    
    Ok(())
}

/// ディレクトリパスを検証する
///
/// 以下の条件をチェックします：
/// - パスが空でないこと
/// - ディレクトリが存在すること（ファイルではないこと）
/// - ディレクトリに読み書き権限があること
///
/// # Arguments
/// * `path` - 検証するディレクトリパス
pub fn validate_directory_path(path: &PathBuf) -> Result<(), BallistaError> {
    trace_fn!("validate_directory_path(path: {})", path.display());
    
    // 共通のパス検証
    validate_path(path, PathType::Directory)?;
    
    // ディレクトリタイプの確認
    if !path.is_dir() {
        return Err(BallistaError::FileError {
            message: "パスはディレクトリではありません".to_string(),
            context_info: format!(": {}", path.display()),
        });
    }
    
    Ok(())
}

/// パス内の環境変数プレースホルダーを展開する
///
/// # 引数
///
/// * `path` - 環境変数プレースホルダーを含むパス文字列
///
/// # 戻り値
///
/// 環境変数が展開されたパス文字列
pub fn expand_env_vars(path: &str) -> String {
    trace_fn!("expand_env_vars(path: {})", path);
    let mut result = path.to_string();
    
    // $HOME プレースホルダーを展開
    if result.contains("$HOME") {
        if let Ok(home_dir) = env::var("HOME") {
            result = result.replace("$HOME", &home_dir);
        } else if let Ok(user_profile) = env::var("USERPROFILE") {
            // Windowsの場合はUSERPROFILEを使用
            result = result.replace("$HOME", &user_profile);
        }
    }
    
    // 必要に応じて他の環境変数も展開することができます
    
    // パスの区切り文字を正規化
    normalize_path(&result)
}
