/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! 設定ファイル監視モジュール
//!
//! このモジュールは、設定ファイルの変更を監視し、
//! 変更があった場合に適切なアクションを実行するための機能を提供します。

use log::{debug, info};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use tauri::{AppHandle, Manager};

use crate::config::{update_config, get_config_file_path};

/// 設定ファイルの監視を開始する
///
/// アプリケーションの設定ファイルに対する変更を監視し、
/// 変更があった場合にイベントをレシーバーに送信します。
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
///
/// # 戻り値
///
/// 監視イベントを受信するためのレシーバー
pub fn start_config_watcher(app_handle: AppHandle) -> Result<Receiver<notify::Result<Event>>, notify::Error> {
    let (tx, rx) = channel();
    
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            tx.send(res).unwrap();
        },
        Config::default(),
    )?;
    
    let config_path = get_config_file_path(&app_handle);
    let config_dir = config_path.parent().unwrap();
    
    info!("設定ファイルの監視を開始します: {}", config_path.display());
    watcher.watch(Path::new(&config_dir), RecursiveMode::NonRecursive)?;
    
    // Watcherをアプリケーションの状態に保存して、スコープ外でもウォッチャーが生きているようにする
    app_handle.manage(watcher);
    
    Ok(rx)
}

/// ファイル変更イベントを処理する
///
/// 監視対象ファイルに変更があった場合、設定を再読み込みし、
/// 必要に応じてアプリケーションの状態を更新します。
///
/// # 引数
///
/// * `event` - 発生したファイル変更イベント
/// * `app_handle` - Tauriアプリケーションのハンドル
pub fn handle_config_changes(event: Event, app_handle: &AppHandle) {
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) => {
            // 監視対象のファイルかチェック
            let config_path = get_config_file_path(app_handle);
            for path in &event.paths {
                if path == &config_path {
                    debug!("設定ファイルが変更されました: {}", path.display());
                    
                    // 設定を再読み込み
                    update_config(app_handle);
                    
                    // ロガーの設定を更新
                    update_logger_settings(app_handle);
                    
                    // フロントエンドにログ更新イベントを送信
                    app_handle.emit_all("log:update", ()).unwrap();
                    
                    break;
                }
            }
        }
        _ => {}
    }
}

/// ロガーの設定を更新する
///
/// 設定ファイルからロガーのレベルと最大ログファイル数を取得し、
/// ロガーの設定を更新します。
///
/// # 引数
///
/// * `app_handle` - Tauriアプリケーションのハンドル
fn update_logger_settings(app_handle: &AppHandle) {
    use crate::config::{get_log_level, load_config};
    use crate::logger::{set_log_level, set_max_log_files};
    
    // 設定を読み込み
    let config = load_config(app_handle);
    
    // ログレベルを更新
    let log_level = get_log_level();
    set_log_level(log_level);
    debug!("ロガーのレベルを更新しました: {:?}", log_level);
    
    // 最大ログファイル数を更新
    let max_files = config.logging.max_files;
    set_max_log_files(max_files);
    debug!("最大ログファイル数を更新しました: {}", max_files);
}
