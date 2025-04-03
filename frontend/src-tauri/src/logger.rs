/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! カスタムロギングシステム
//!
//! このモジュールは、アプリケーション全体で使用されるカスタムロギング機能を提供します。
//! ログはファイルとコンソールの両方に出力され、ログレベルやログファイルのローテーションなどを
//! 設定できます。

use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// 定数定義
const LATEST_LOG_FILENAME: &str = "latest.log";

// グローバル変数
static LOG_LEVEL: Mutex<LevelFilter> = Mutex::new(LevelFilter::Info);
static MAX_LOG_FILES: Mutex<usize> = Mutex::new(10);  // 保持するログファイルの最大数（デフォルト10）

/// カスタムロガー構造体
///
/// アプリケーション固有のロギング要件に対応するカスタムロガー。
/// ログファイルとlatest.logの両方にログを出力します。
pub struct CustomLogger {
    /// メインログファイルのパス
    log_file: Mutex<Option<PathBuf>>,
    /// 最新のログファイル（latest.log）のパス
    latest_file: Mutex<Option<PathBuf>>,
    /// ログディレクトリのパス
    log_dir: Mutex<Option<PathBuf>>,
}

impl CustomLogger {
    /// 新しいカスタムロガーを作成する
    ///
    /// # 引数
    ///
    /// * `log_file` - メインログファイルのパス（オプション）
    /// * `log_dir` - ログディレクトリのパス（オプション）
    ///
    /// # 戻り値
    ///
    /// 設定されたカスタムロガー
    pub fn new(log_file: Option<PathBuf>, log_dir: Option<PathBuf>) -> Self {
        let latest_file = log_dir.as_ref().map(|dir| dir.join(LATEST_LOG_FILENAME));
        
        // 初期化時にlatest.logを作成（既存なら削除）
        if let Some(latest_path) = &latest_file {
            // 既存のファイルを削除
            if latest_path.exists() {
                println!("[Logger] 既存のlatest.logを削除します: {}", latest_path.display());
                if let Err(e) = fs::remove_file(latest_path) {
                    eprintln!("[Logger] latest.logの削除に失敗: {}", e);
                }
            }
            
            // 空のファイルを作成
            if let Err(e) = File::create(latest_path) {
                eprintln!("[Logger] latest.logの作成に失敗: {}", e);
            } else {
                println!("[Logger] 新しいlatest.logを作成しました: {}", latest_path.display());
            }
        }
        
        Self {
            log_file: Mutex::new(log_file),
            latest_file: Mutex::new(latest_file),
            log_dir: Mutex::new(log_dir),
        }
    }
    
    /// ログファイルのパスを設定する
    ///
    /// # 引数
    ///
    /// * `path` - 設定するログファイルのパス
    pub fn set_log_file(&self, path: PathBuf) {
        if let Ok(mut log_file) = self.log_file.lock() {
            *log_file = Some(path);
        }
    }
    
    /// ログディレクトリを設定する
    ///
    /// # 引数
    ///
    /// * `dir` - 設定するログディレクトリのパス
    pub fn set_log_dir(&self, dir: PathBuf) {
        // ディレクトリを設定
        if let Ok(mut log_dir) = self.log_dir.lock() {
            *log_dir = Some(dir.clone());
        }
        
        // latest.logのパスを更新
        let latest_path = dir.join(LATEST_LOG_FILENAME);
        if let Ok(mut latest_file) = self.latest_file.lock() {
            *latest_file = Some(latest_path);
        }
    }
}

impl Log for CustomLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if let Ok(level) = LOG_LEVEL.lock() {
            metadata.level() <= *level
        } else {
            metadata.level() <= LevelFilter::Info
        }
    }
    
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level = record.level();
        let target = record.target();
        let args = record.args();
        
        // ログメッセージの整形
        // ファイル名と行番号を追加
        let file = record.file().unwrap_or("unknown");
        let line = record.line().unwrap_or(0);
        let message = format!("{} [{}] {}({}:{}): {}\n", now, level, target, file, line, args);
        
        // ターミナルに出力
        print!("{}", message);
        
        // 通常のログファイルに書き込む
        if let Ok(log_file) = self.log_file.lock() {
            if let Some(path) = &*log_file {
                // ディレクトリが存在しない場合は作成
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        let _ = fs::create_dir_all(parent);
                    }
                }
                
                // ファイルに追記
                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path) {
                    if let Err(e) = file.write_all(message.as_bytes()) {
                        eprintln!("[Logger] ログファイルへの書き込みに失敗: {}", e);
                    }
                }
            }
        }
        
        // latest.logにも書き込む
        if let Ok(latest_file) = self.latest_file.lock() {
            if let Some(path) = &*latest_file {
                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path) {
                    if let Err(e) = file.write_all(message.as_bytes()) {
                        eprintln!("[Logger] latest.logへの書き込みに失敗: {}", e);
                    }
                }
            }
        }
    }
    
    fn flush(&self) {
        // ターミナル出力をフラッシュ
        let _ = std::io::stdout().flush();
        
        // 通常ログファイル出力をフラッシュ
        if let Ok(log_file) = self.log_file.lock() {
            if let Some(path) = &*log_file {
                if let Ok(file) = File::open(path) {
                    let _ = file.sync_all();
                }
            }
        }
        
        // latest.logをフラッシュ
        if let Ok(latest_file) = self.latest_file.lock() {
            if let Some(path) = &*latest_file {
                if let Ok(file) = File::open(path) {
                    let _ = file.sync_all();
                }
            }
        }
    }
}

/// 保持するログファイルの最大数を設定する
///
/// # 引数
///
/// * `max_files` - 設定する最大ファイル数
pub fn set_max_log_files(max_files: usize) {
    if let Ok(mut current_max) = MAX_LOG_FILES.lock() {
        *current_max = max_files;
        println!("[Logger] 最大ログファイル数を設定しました: {}", max_files);
    }
}

/// 現在の最大ログファイル数を取得する
///
/// # 戻り値
///
/// 現在設定されている最大ログファイル数
pub fn get_max_log_files() -> usize {
    if let Ok(current_max) = MAX_LOG_FILES.lock() {
        *current_max
    } else {
        10 // デフォルト値
    }
}

// 古いログファイルを管理する（削除したファイル数を返す）
fn manage_log_files(log_dir: &Path) -> std::io::Result<usize> {
    println!("[Logger] 古いログファイルを管理します: {}", log_dir.display());
    
    // ログディレクトリ内のファイル一覧を取得
    let entries = match fs::read_dir(log_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("[Logger] ディレクトリの読み取りに失敗: {}", e);
            return Err(e);
        }
    };
    
    // 通常のログファイルをフィルタリング
    let mut log_files: Vec<PathBuf> = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && 
           path.extension().map_or(false, |ext| ext == "log") && 
           path.file_name().map_or(false, |name| !name.to_string_lossy().contains("latest")) {
            log_files.push(path);
        }
    }
    
    println!("[Logger] 検出したログファイル数: {}", log_files.len());
    
    if log_files.is_empty() {
        return Ok(0);
    }
    
    // 最終更新日時が古い順にソート
    log_files.sort_by(|a, b| {
        let a_modified = fs::metadata(a).and_then(|m| m.modified()).unwrap_or_else(|_| std::time::SystemTime::now());
        let b_modified = fs::metadata(b).and_then(|m| m.modified()).unwrap_or_else(|_| std::time::SystemTime::now());
        a_modified.cmp(&b_modified)
    });
    
    // 削除したファイル数
    let mut deleted_count = 0;
    
    // 現在の最大ログファイル数を取得
    let max_files = get_max_log_files();
    
    // 最大数を超えた古いファイルを削除
    if log_files.len() > max_files {
        let to_remove = log_files.len() - max_files;
        println!("[Logger] 削除対象ファイル数: {} (最大保持数: {})", to_remove, max_files);
        
        for file in log_files.iter().take(to_remove) {
            println!("[Logger] 削除: {}", file.display());
            if let Err(e) = fs::remove_file(file) {
                eprintln!("[Logger] ファイル削除エラー: {}: {}", file.display(), e);
            } else {
                deleted_count += 1;
            }
        }
    }
    
    Ok(deleted_count)
}

// ロガーのシングルトンインスタンスを保持する静的変数
static LOGGER: once_cell::sync::Lazy<CustomLogger> = once_cell::sync::Lazy::new(|| {
    CustomLogger::new(None, None)
});

/// ロギングシステムを初期化する
///
/// グローバルなロガーを設定し、指定されたファイルにログを出力できるようにする。
///
/// # 引数
///
/// * `log_file` - メインログファイルのパス（オプション）
///
/// # 戻り値
///
/// 初期化に成功した場合はOk、失敗した場合はエラー
pub fn init_logger(log_file: Option<PathBuf>) -> Result<(), SetLoggerError> {
    // ログディレクトリを取得
    let log_dir = log_file.as_ref().and_then(|p| p.parent().map(|p| p.to_path_buf()));
    
    // ログディレクトリを設定
    if let Some(dir) = &log_dir {
        // latest.logを確実にクリア（既存なら削除してから新規作成）
        let latest_path = dir.join(LATEST_LOG_FILENAME);
        if latest_path.exists() {
            println!("[Logger] 既存のlatest.logを削除します: {}", latest_path.display());
            if let Err(e) = fs::remove_file(&latest_path) {
                eprintln!("[Logger] latest.logの削除に失敗: {}", e);
            }
        }
        
        // 空のlatest.logを作成
        if let Err(e) = File::create(&latest_path) {
            eprintln!("[Logger] latest.logの作成に失敗: {}", e);
        } else {
            println!("[Logger] 新しいlatest.logを作成しました: {}", latest_path.display());
        }
        
        // ロガーにディレクトリを設定
        LOGGER.set_log_dir(dir.clone());
        
        // 古いログファイルを管理
        if let Err(e) = manage_log_files(dir) {
            eprintln!("[Logger] ログファイル管理に失敗: {}", e);
        }
    }
    
    // ログファイルを設定
    if let Some(path) = log_file {
        LOGGER.set_log_file(path);
    }
    
    log::set_logger(&*LOGGER).map(|()| {
        set_log_level(LevelFilter::Info);
        println!("[Logger] ロガーを初期化しました");
    })
}

/// グローバルなログレベルを設定する
///
/// アプリケーション全体で使用されるログレベルを変更する。
///
/// # 引数
///
/// * `level` - 設定するログレベル
pub fn set_log_level(level: LevelFilter) {
    if let Ok(mut current_level) = LOG_LEVEL.lock() {
        *current_level = level;
        log::set_max_level(level);
        println!("[Logger] ログレベルを設定しました: {:?}", level);
    }
}
