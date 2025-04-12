use std::io;
use thiserror::Error;
use log::{error, warn};
use serde::{Serialize, Deserialize};

/// エラー型
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum BallistaError {
    #[error("ファイル操作エラー: {0}")]
    FileError(String),
    
    #[error("XML解析エラー: {0}")]
    XmlError(String),
    
    #[error("バックアップエラー: {0}")]
    BackupError(String),
    
    #[error("プリセットエラー: {0}")]
    PresetError(String),
    
    #[error("設定エラー: {0}")]
    ConfigError(String),
    
    #[error("内部エラー: {0}")]
    InternalError(String),
}

/// エラーの重大度
pub enum Severity {
    Low,    // 影響の少ないエラー
    Medium, // 一部機能が使えないエラー
    High,   // アプリケーションの主要機能が使えないエラー
    Critical, // アプリケーションの動作が停止するようなエラー
}

impl BallistaError {
    /// エラーの重大度を取得
    pub fn severity(&self) -> Severity {
        match self {
            BallistaError::FileError(_) => Severity::Medium,
            BallistaError::XmlError(_) => Severity::High,
            BallistaError::BackupError(_) => Severity::Low,
            BallistaError::PresetError(_) => Severity::Low,
            BallistaError::ConfigError(_) => Severity::Medium,
            BallistaError::InternalError(_) => Severity::Critical,
        }
    }
    
    /// エラーメッセージを取得（ユーザーフレンドリーなメッセージ）
    pub fn user_message(&self) -> String {
        match self {
            BallistaError::FileError(msg) => format!("ファイルの処理中にエラーが発生しました: {}", msg),
            BallistaError::XmlError(msg) => format!("XMLデータの処理中にエラーが発生しました: {}", msg),
            BallistaError::BackupError(msg) => format!("バックアップの処理中にエラーが発生しました: {}", msg),
            BallistaError::PresetError(msg) => format!("プリセットの処理中にエラーが発生しました: {}", msg),
            BallistaError::ConfigError(msg) => format!("設定の処理中にエラーが発生しました: {}", msg),
            BallistaError::InternalError(msg) => format!("内部エラーが発生しました: {}", msg),
        }
    }
    
    /// エラーログを出力
    pub fn log(&self) {
        match self.severity() {
            Severity::Low => warn!("{}", self),
            Severity::Medium => warn!("{}", self),
            Severity::High => error!("{}", self),
            Severity::Critical => error!("致命的なエラー: {}", self),
        }
    }
}

// BallistaErrorからStringへの変換（APIレイヤーで使用）
impl From<BallistaError> for String {
    fn from(err: BallistaError) -> Self {
        err.to_string()
    }
}

// 標準エラーからのコンバージョン
impl From<io::Error> for BallistaError {
    fn from(err: io::Error) -> Self {
        BallistaError::FileError(err.to_string())
    }
}

impl From<quick_xml::Error> for BallistaError {
    fn from(err: quick_xml::Error) -> Self {
        BallistaError::XmlError(err.to_string())
    }
}

impl From<serde_json::Error> for BallistaError {
    fn from(err: serde_json::Error) -> Self {
        BallistaError::PresetError(err.to_string())
    }
}

impl From<String> for BallistaError {
    fn from(err: String) -> Self {
        if err.contains("XML") || err.contains("xml") {
            BallistaError::XmlError(err)
        } else if err.contains("ファイル") || err.contains("file") {
            BallistaError::FileError(err)
        } else if err.contains("バックアップ") || err.contains("backup") {
            BallistaError::BackupError(err)
        } else if err.contains("プリセット") || err.contains("preset") {
            BallistaError::PresetError(err)
        } else if err.contains("設定") || err.contains("config") {
            BallistaError::ConfigError(err)
        } else {
            BallistaError::InternalError(err)
        }
    }
}
