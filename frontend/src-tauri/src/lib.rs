/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Ballistaアプリケーションのコアライブラリ

// マクロをエクスポートして外部クレートからも使用できるようにする
#[macro_export]
macro_rules! trace_fn {
    ($msg:expr) => {
        log::trace!("[TRACE] {}", $msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        log::trace!("[TRACE] {}", format!($fmt, $($arg)*));
    };
}

#[macro_use]
pub mod logger;

pub mod error;
pub mod config;

// ファイル監視システム
pub mod watcher;

// コアモジュール
pub mod core {
    // モディングXML処理モジュール
    pub mod modding;
    
    // バックアップ機能モジュール
    pub mod backup;
    
    // プリセット機能モジュール
    pub mod preset;
}

#[cfg(test)]
pub mod tests;