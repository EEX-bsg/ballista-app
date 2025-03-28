/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! プリセット機能のテスト

use crate::core::preset;
use crate::core::modding::{ModInfo, ModdingXmlData};
use crate::tests::common;
use std::fs;
use std::path::Path;

/// プリセット保存テスト
#[test]
fn test_preset_save() {
    // テスト環境のセットアップ
    let temp_dir = common::setup_test_environment();
    
    // テスト用のデータを作成
    let data = ModdingXmlData {
        enabled_mods: vec![
            ModInfo {
                uuid: "test-uuid-1".to_string(),
                name: "Test Mod 1".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
                source: crate::core::modding::Source::Workshop,
                workshop_id: Some("1234567890".to_string()),
            }
        ],
        disabled_mods: vec![],
        game_version: Some("1.0.0".to_string()),
        file_path: None,
    };
    
    // プリセットを保存
    let preset_path = preset::save_preset("TestPreset", Some("Test description"), &data).unwrap();
    
    // プリセットが作成されたことを確認
    assert!(preset_path.exists());
    
    // テスト環境のクリーンアップ
    common::cleanup_test_environment(&temp_dir);
}

/// プリセット一覧取得テスト
#[test]
fn test_preset_list() {
    // テスト環境のセットアップ
    let temp_dir = common::setup_test_environment();
    
    // テスト用のデータを作成
    let data = ModdingXmlData {
        enabled_mods: vec![],
        disabled_mods: vec![],
        game_version: Some("1.0.0".to_string()),
        file_path: None,
    };
    
    // プリセットを2つ保存
    let preset_path1 = preset::save_preset("TestPreset1", None, &data).unwrap();
    let preset_path2 = preset::save_preset("TestPreset2", None, &data).unwrap();
    
    // プリセット一覧を取得
    let preset_list = preset::get_preset_list().unwrap();
    
    // プリセットが2つ以上あることを確認
    assert!(preset_list.len() >= 2);
    
    // プリセットが存在することを確認
    assert!(preset_path1.exists());
    assert!(preset_path2.exists());
    
    // テスト環境のクリーンアップ
    common::cleanup_test_environment(&temp_dir);
}

/// プリセット読み込みテスト
#[test]
fn test_preset_load() {
    // テスト環境のセットアップ
    let temp_dir = common::setup_test_environment();
    
    // テスト用プリセットJSONを作成
    let preset_json = common::generate_basic_preset_json();
    let preset_path = common::create_test_file(&temp_dir, "test_preset.json", &preset_json);
    
    // プリセットを読み込み
    let preset_data = preset::load_preset(&preset_path).unwrap();
    
    // プリセットの基本情報を確認
    assert_eq!(preset_data.name, "TestPreset");
    assert_eq!(preset_data.description.unwrap(), "テストプリセットの説明");
    
    // プリセットのModリストを確認
    assert_eq!(preset_data.mods.len(), 2);
    
    // テスト環境のクリーンアップ
    common::cleanup_test_environment(&temp_dir);
}

/// プリセット適用テスト
#[test]
fn test_preset_apply() {
    // テスト環境のセットアップ
    let temp_dir = common::setup_test_environment();
    
    // テスト用XMLを作成
    let xml_content = common::generate_basic_modding_xml();
    let xml_path = common::create_test_file(&temp_dir, "modding.xml", &xml_content);
    
    // テスト用プリセットJSONを作成
    let preset_json = common::generate_basic_preset_json();
    let preset_path = common::create_test_file(&temp_dir, "test_preset.json", &preset_json);
    
    // プリセットを適用
    preset::apply_preset(&preset_path, &xml_path).unwrap();
    
    // XMLが更新されたことを確認
    let updated_xml = fs::read_to_string(&xml_path).unwrap();
    assert!(updated_xml.contains("TestMod1"));
    
    // テスト環境のクリーンアップ
    common::cleanup_test_environment(&temp_dir);
}

/// プリセット削除テスト
#[test]
fn test_preset_delete() {
    // テスト環境のセットアップ
    let temp_dir = common::setup_test_environment();
    
    // テスト用のデータを作成
    let data = ModdingXmlData {
        enabled_mods: vec![],
        disabled_mods: vec![],
        game_version: Some("1.0.0".to_string()),
        file_path: None,
    };
    
    // プリセットを保存
    let preset_path = preset::save_preset("TestPreset", None, &data).unwrap();
    
    // プリセットが作成されたことを確認
    assert!(preset_path.exists());
    
    // プリセットを削除
    preset::delete_preset(&preset_path).unwrap();
    
    // プリセットが削除されたことを確認
    assert!(!preset_path.exists());
    
    // テスト環境のクリーンアップ
    common::cleanup_test_environment(&temp_dir);
} 