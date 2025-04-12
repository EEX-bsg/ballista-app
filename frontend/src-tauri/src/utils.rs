/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities
//!
//! 複数のファイルで利用する便利関数などを置く場所

use std::path::{Path, PathBuf};
use std::fs;
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
fn validate_path(path: &Path, path_type: PathType) -> Result<(), BallistaError> {
    // パスの形式チェック
    if path.to_string_lossy().is_empty() {
        return Err(BallistaError::FileError("パスが空です".to_string()));
    }
    
    // 存在確認
    if !path.exists() {
        return Err(BallistaError::FileError(
            format!("{}が存在しません: {}", path_type.display_name(), path.display())
        ));
    }
    
    // メタデータによる権限の確認
    match fs::metadata(path) {
        Ok(metadata) => {
            // 読み取り専用かどうかチェック
            if metadata.permissions().readonly() {
                return Err(BallistaError::FileError(
                    format!("{}は読み取り専用です: {}", path_type.display_name(), path.display())
                ));
            }
        },
        Err(e) => return Err(BallistaError::FileError(
            format!("{}のメタデータの取得に失敗しました: {}", path_type.display_name(), e)
        )),
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
pub fn validate_file_path(path: &Path) -> Result<(), BallistaError> {
    trace_fn!("validate_file_path(path: {})", path.display());
    
    // 親ディレクトリの存在確認
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(BallistaError::FileError(
                format!("親ディレクトリが存在しません: {}", parent.display())
            ));
        }
    }
    
    // 共通のパス検証
    validate_path(path, PathType::File)?;
    
    // ファイルタイプの確認
    if !path.is_file() {
        return Err(BallistaError::FileError(
            format!("パスはファイルではありません: {}", path.display())
        ));
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
pub fn validate_directory_path(path: &Path) -> Result<(), BallistaError> {
    trace_fn!("validate_directory_path(path: {})", path.display());
    
    // 共通のパス検証
    validate_path(path, PathType::Directory)?;
    
    // ディレクトリタイプの確認
    if !path.is_dir() {
        return Err(BallistaError::FileError(
            format!("パスはディレクトリではありません: {}", path.display())
        ));
    }
    
    Ok(())
}
