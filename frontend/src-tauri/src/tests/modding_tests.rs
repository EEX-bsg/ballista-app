/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Modding XMLのテスト

use crate::core::modding::{self, ModInfo, ModdingXmlData};
use crate::config;
use crate::tests::common;
use std::fs;
use std::path::Path;
use std::io::Write;
use tempfile::tempdir;

/// アプリ設定のテスト
#[test]
fn test_app_config() {
    // 初期の設定を読み込み
    let config = config::APP_CONFIG.lock().unwrap();
    assert!(!config.game_path.is_empty());
    assert!(!config.workshop_path.is_empty());
}

/// パス検証のテスト
#[test]
fn test_path_validation() {
    // 一時ディレクトリを作成（存在するパスのテスト用）
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().to_string_lossy().to_string();
    
    // 存在するディレクトリの検証
    let result = modding::validate_path(&temp_path);
    assert!(result.is_ok());
    
    // 存在しないディレクトリの検証
    let non_existent_path = "/path/that/does/not/exist";
    let result = modding::validate_path(non_existent_path);
    assert!(result.is_err());
}

/// XML解析のテスト
#[test]
fn test_xml_parsing() {
    let temp_dir = tempdir().unwrap();
    let xml_path = temp_dir.path().join("ModIO.xml");
    
    // テスト用XMLファイルを作成
    let xml_content = r#"<?xml version="1.0" encoding="utf-8"?>
<ModManifest>
  <GameVersion>1.2.3</GameVersion>
  <ModInfos>
    <ModInfo>
      <UUID>mod1-uuid</UUID>
      <Name>Test Mod 1</Name>
      <Version>1.0.0</Version>
      <Source>Workshop</Source>
      <WorkshopID>1234567890</WorkshopID>
      <Enabled>true</Enabled>
    </ModInfo>
  </ModInfos>
</ModManifest>"#;
    
    fs::write(&xml_path, xml_content).unwrap();
    
    // XMLの読み込みテスト
    let result = modding::read_modding_xml(&xml_path.to_string_lossy().to_string());
    assert!(result.is_ok());
    
    let data = result.unwrap();
    assert_eq!(data.game_version, Some("1.2.3".to_string()));
    assert_eq!(data.mods.len(), 1);
    
    let mod1 = &data.mods[0];
    assert_eq!(mod1.uuid, "mod1-uuid");
    assert_eq!(mod1.name, "Test Mod 1");
    assert_eq!(mod1.version, "1.0.0");
    assert_eq!(mod1.source, modding::ModSource::Workshop);
    assert_eq!(mod1.workshop_id, Some("1234567890".to_string()));
    assert_eq!(mod1.enabled, true);
}

/// XML生成のテスト
#[test]
fn test_xml_generation() {
    let temp_dir = tempdir().unwrap();
    let xml_path = temp_dir.path().join("ModIO.xml");
    
    // ModdingXmlDataの作成
    let mut data = ModdingXmlData {
        game_version: Some("1.2.3".to_string()),
        mods: vec![
            ModInfo {
                uuid: "mod1-uuid".to_string(),
                name: "Test Mod 1".to_string(),
                version: "1.0.0".to_string(),
                source: modding::ModSource::Workshop,
                workshop_id: Some("1234567890".to_string()),
                enabled: true,
            },
            ModInfo {
                uuid: "mod2-uuid".to_string(),
                name: "Test Mod 2".to_string(),
                version: "2.0.0".to_string(),
                source: modding::ModSource::Workshop,
                workshop_id: Some("2345678901".to_string()),
                enabled: false,
            }
        ],
    };
    
    // XMLの書き込みテスト
    let result = modding::write_modding_xml(&data, &xml_path.to_string_lossy().to_string());
    assert!(result.is_ok());
    
    // 書き込んだXMLの読み込みテスト
    let result = modding::read_modding_xml(&xml_path.to_string_lossy().to_string());
    assert!(result.is_ok());
    
    let read_data = result.unwrap();
    assert_eq!(read_data.game_version, Some("1.2.3".to_string()));
    assert_eq!(read_data.mods.len(), 2);
    
    // モッドの有効/無効切り替えテスト
    let mod_uuid = "mod2-uuid";
    modding::toggle_mod_enabled(&mut data, mod_uuid);
    
    let mod2 = data.mods.iter().find(|m| m.uuid == mod_uuid).unwrap();
    assert_eq!(mod2.enabled, true);
} 