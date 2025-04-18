use std::io;
use thiserror::Error;
use log::{error, warn};
use serde::{Serialize, Deserialize, Serializer};

/// エラー型
#[derive(Error, Debug, Deserialize)]
pub enum BallistaError {
    #[error("ファイル操作エラー: {message}{context_info}")]
    FileError {
        message: String,
        context_info: String,
    },
    
    #[error("XML解析エラー: {message}{context_info}")]
    XmlError {
        message: String,
        context_info: String,
    },
    
    #[error("バックアップエラー: {message}{context_info}")]
    BackupError {
        message: String,
        context_info: String,
    },
    
    #[error("プリセットエラー: {message}{context_info}")]
    PresetError {
        message: String,
        context_info: String,
    },
    
    #[error("設定エラー: {message}{context_info}")]
    ConfigError {
        message: String,
        context_info: String,
    },
    
    #[error("内部エラー: {message}{context_info}")]
    InternalError {
        message: String,
        context_info: String,
    },
}

// カスタムシリアライズ実装
// これによりTypescript側で直接エラーメッセージにアクセスできるようにする
impl Serialize for BallistaError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Tauriにエラーメッセージを文字列として渡す
        serializer.serialize_str(&self.user_message())
    }
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
            BallistaError::FileError { .. } => Severity::Medium,
            BallistaError::XmlError { .. } => Severity::High,
            BallistaError::BackupError { .. } => Severity::Low,
            BallistaError::PresetError { .. } => Severity::Low,
            BallistaError::ConfigError { .. } => Severity::Medium,
            BallistaError::InternalError { .. } => Severity::Critical,
        }
    }
    
    /// エラーの基本メッセージのみを取得
    pub fn base_message(&self) -> String {
        match self {
            BallistaError::FileError { message, .. } => message.clone(),
            BallistaError::XmlError { message, .. } => message.clone(),
            BallistaError::BackupError { message, .. } => message.clone(),
            BallistaError::PresetError { message, .. } => message.clone(),
            BallistaError::ConfigError { message, .. } => message.clone(),
            BallistaError::InternalError { message, .. } => message.clone(),
        }
    }
    
    /// エラーコンテキスト情報を取得
    pub fn context_info(&self) -> String {
        match self {
            BallistaError::FileError { context_info, .. } => context_info.clone(),
            BallistaError::XmlError { context_info, .. } => context_info.clone(),
            BallistaError::BackupError { context_info, .. } => context_info.clone(),
            BallistaError::PresetError { context_info, .. } => context_info.clone(),
            BallistaError::ConfigError { context_info, .. } => context_info.clone(),
            BallistaError::InternalError { context_info, .. } => context_info.clone(),
        }
    }
    
    /// エラーメッセージを取得（ユーザーフレンドリーなメッセージ）
    pub fn user_message(&self) -> String {
        self.base_message()
    }
    
    /// エラーログを出力（詳細情報）
    pub fn log(&self) {
        let full_message = match self {
            BallistaError::FileError { message, context_info } => 
                format!("ファイルの処理中にエラーが発生しました: {}{}", message, context_info),
            BallistaError::XmlError { message, context_info } => 
                format!("XMLデータの処理中にエラーが発生しました: {}{}", message, context_info),
            BallistaError::BackupError { message, context_info } => 
                format!("バックアップの処理中にエラーが発生しました: {}{}", message, context_info),
            BallistaError::PresetError { message, context_info } => 
                format!("プリセットの処理中にエラーが発生しました: {}{}", message, context_info),
            BallistaError::ConfigError { message, context_info } => 
                format!("設定の処理中にエラーが発生しました: {}{}", message, context_info),
            BallistaError::InternalError { message, context_info } => 
                format!("内部エラーが発生しました: {}{}", message, context_info),
        };
        
        match self.severity() {
            Severity::Low => warn!("{}", full_message),
            Severity::Medium => warn!("{}", full_message),
            Severity::High => error!("{}", full_message),
            Severity::Critical => error!("致命的なエラー: {}", full_message),
        }
    }
    
    /// コンテキスト情報を追加
    pub fn add_context(self, context: &str) -> Self {
        match self {
            BallistaError::FileError { message, mut context_info } => {
                if !context_info.is_empty() {
                    context_info.push_str(" | ");
                }
                context_info.push_str(context);
                BallistaError::FileError { message, context_info }
            },
            BallistaError::XmlError { message, mut context_info } => {
                if !context_info.is_empty() {
                    context_info.push_str(" | ");
                }
                context_info.push_str(context);
                BallistaError::XmlError { message, context_info }
            },
            BallistaError::BackupError { message, mut context_info } => {
                if !context_info.is_empty() {
                    context_info.push_str(" | ");
                }
                context_info.push_str(context);
                BallistaError::BackupError { message, context_info }
            },
            BallistaError::PresetError { message, mut context_info } => {
                if !context_info.is_empty() {
                    context_info.push_str(" | ");
                }
                context_info.push_str(context);
                BallistaError::PresetError { message, context_info }
            },
            BallistaError::ConfigError { message, mut context_info } => {
                if !context_info.is_empty() {
                    context_info.push_str(" | ");
                }
                context_info.push_str(context);
                BallistaError::ConfigError { message, context_info }
            },
            BallistaError::InternalError { message, mut context_info } => {
                if !context_info.is_empty() {
                    context_info.push_str(" | ");
                }
                context_info.push_str(context);
                BallistaError::InternalError { message, context_info }
            },
        }
    }
}

// BallistaErrorからStringへの変換（APIレイヤーで使用）
impl From<BallistaError> for String {
    fn from(err: BallistaError) -> Self {
        err.user_message()
    }
}

// 標準エラーからのコンバージョン
impl From<io::Error> for BallistaError {
    fn from(err: io::Error) -> Self {
        BallistaError::FileError { 
            message: err.to_string(), 
            context_info: String::new() 
        }
    }
}

impl From<quick_xml::Error> for BallistaError {
    fn from(err: quick_xml::Error) -> Self {
        BallistaError::XmlError { 
            message: err.to_string(), 
            context_info: String::new() 
        }
    }
}

impl From<serde_json::Error> for BallistaError {
    fn from(err: serde_json::Error) -> Self {
        BallistaError::PresetError { 
            message: err.to_string(), 
            context_info: String::new() 
        }
    }
}

impl From<String> for BallistaError {
    fn from(err: String) -> Self {
        if err.contains("XML") || err.contains("xml") {
            BallistaError::XmlError { message: err, context_info: String::new() }
        } else if err.contains("ファイル") || err.contains("file") {
            BallistaError::FileError { message: err, context_info: String::new() }
        } else if err.contains("バックアップ") || err.contains("backup") {
            BallistaError::BackupError { message: err, context_info: String::new() }
        } else if err.contains("プリセット") || err.contains("preset") {
            BallistaError::PresetError { message: err, context_info: String::new() }
        } else if err.contains("設定") || err.contains("config") {
            BallistaError::ConfigError { message: err, context_info: String::new() }
        } else {
            BallistaError::InternalError { message: err, context_info: String::new() }
        }
    }
}
