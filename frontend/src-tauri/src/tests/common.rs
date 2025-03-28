/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! テスト用共通関数・ユーティリティ

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

/// テスト環境のセットアップを行う
/// 一時ディレクトリを作成し、テストデータを初期化する
pub fn setup_test_environment() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let pid = process::id();
    
    let temp_dir = std::env::temp_dir()
        .join(format!("ballista_test_{}_{}", timestamp, pid));
    
    fs::create_dir_all(&temp_dir).expect("一時ディレクトリの作成に失敗しました");
    
    temp_dir
}

/// テスト環境のクリーンアップを行う
/// テスト終了後に一時ディレクトリを削除する
pub fn cleanup_test_environment(temp_dir: &Path) {
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(temp_dir);
    }
}

/// テストファイルを作成する
pub fn create_test_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
    let file_path = dir.join(filename);
    fs::write(&file_path, content).expect("テストファイルの作成に失敗しました");
    file_path
}

/// 基本的なModding.xmlのコンテンツを生成する
pub fn generate_basic_modding_xml() -> String {
    r#"<?xml version="1.0" encoding="utf-8"?>
<ModdingInfo xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <EnabledMods>
    <ModInfo>
      <uuid>12345678-1234-5678-1234-567812345678</uuid>
      <name>TestMod1</name>
      <version>1.0.0</version>
      <enabled>true</enabled>
      <modtype>Workshop</modtype>
      <workshopId>1234567890</workshopId>
    </ModInfo>
    <ModInfo>
      <uuid>22345678-1234-5678-1234-567812345678</uuid>
      <name>TestMod2</name>
      <version>1.0.1</version>
      <enabled>true</enabled>
      <modtype>Local</modtype>
    </ModInfo>
  </EnabledMods>
  <DisabledMods>
    <ModInfo>
      <uuid>32345678-1234-5678-1234-567812345678</uuid>
      <name>TestMod3</name>
      <version>2.0.0</version>
      <enabled>false</enabled>
      <modtype>Workshop</modtype>
      <workshopId>2345678901</workshopId>
    </ModInfo>
  </DisabledMods>
  <GameVersion>1.2.3</GameVersion>
</ModdingInfo>"#.to_string()
}

/// 基本的なプリセットJSONのコンテンツを生成する
pub fn generate_basic_preset_json() -> String {
    r#"{
  "name": "TestPreset",
  "description": "テストプリセットの説明",
  "created_at": "2023-03-01T12:00:00Z",
  "modding_data": {
    "enabled_mods": [
      {
        "uuid": "12345678-1234-5678-1234-567812345678",
        "name": "TestMod1",
        "version": "1.0.0",
        "enabled": true,
        "modtype": "Workshop",
        "workshopId": 1234567890
      }
    ],
    "disabled_mods": [
      {
        "uuid": "32345678-1234-5678-1234-567812345678",
        "name": "TestMod3",
        "version": "2.0.0",
        "enabled": false,
        "modtype": "Workshop",
        "workshopId": 2345678901
      }
    ],
    "game_version": "1.2.3"
  }
}"#.to_string()
} 