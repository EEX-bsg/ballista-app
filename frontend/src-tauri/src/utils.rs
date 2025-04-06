/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities
//!
//! 複数のファイルで利用する便利関数などを置く場所

use std::path::{Path, PathBuf};
use std::fs;
use crate::trace_fn;

/// パスが有効かどうかを検証する
pub fn validate_path(path: &Path) -> Result<(), String> {
    trace_fn!("validate_path(path: {})", path.display());
    
    // パスの形式チェック
    if path.to_string_lossy().is_empty() {
        return Err("パスが空です".to_string());
    }
    
    // ディレクトリの存在確認
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(format!("親ディレクトリが存在しません: {}", parent.display()));
        }
    }
    
    // ファイルの存在確認（存在する場合はアクセス権限確認）
    if path.exists() {
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.permissions().readonly() && !path.to_string_lossy().contains(".bak") {
                    return Err(format!("ファイルは読み取り専用です: {}", path.display()));
                }
            }
            Err(e) => return Err(format!("ファイルのメタデータの取得に失敗しました: {}", e)),
        }
    }
    
    Ok(())
}