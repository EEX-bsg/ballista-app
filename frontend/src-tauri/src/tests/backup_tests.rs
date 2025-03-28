/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! バックアップ機能のテスト

use crate::core::backup;
use crate::tests::common;
use std::fs;
use std::path::Path;

/// バックアップ作成テスト
#[test]
fn test_backup_creation() {
    // テスト環境のセットアップ
    let temp_dir = common::setup_test_environment();
    
    // テストファイルを作成
    let test_file_path = common::create_test_file(&temp_dir, "test.xml", "test content");
    
    // バックアップを作成
    let backup_path = backup::create_backup(&test_file_path).unwrap();
    
    // バックアップが作成されたことを確認
    assert!(backup_path.exists());
    
    // バックアップの内容が元のファイルと同じであることを確認
    let original_content = fs::read_to_string(&test_file_path).unwrap();
    let backup_content = fs::read_to_string(&backup_path).unwrap();
    assert_eq!(original_content, backup_content);
    
    // テスト環境のクリーンアップ
    common::cleanup_test_environment(&temp_dir);
}

/// バックアップ一覧取得テスト
#[test]
fn test_backup_list() {
    // テスト環境のセットアップ
    let temp_dir = common::setup_test_environment();
    
    // テストファイルを作成
    let test_file_path = common::create_test_file(&temp_dir, "test.xml", "test content");
    
    // バックアップを作成
    let backup_path1 = backup::create_backup(&test_file_path).unwrap();
    let backup_path2 = backup::create_backup(&test_file_path).unwrap();
    
    // バックアップ一覧を取得
    let backup_list = backup::get_backup_list(&test_file_path).unwrap();
    
    // バックアップが2つあることを確認
    assert_eq!(backup_list.len(), 2);
    
    // 各バックアップの存在を確認
    assert!(backup_path1.exists());
    assert!(backup_path2.exists());
    
    // テスト環境のクリーンアップ
    common::cleanup_test_environment(&temp_dir);
}

/// バックアップ復元テスト
#[test]
fn test_backup_restore() {
    // テスト環境のセットアップ
    let temp_dir = common::setup_test_environment();
    
    // テストファイルを作成
    let test_file_path = common::create_test_file(&temp_dir, "test.xml", "original content");
    
    // バックアップを作成
    let backup_path = backup::create_backup(&test_file_path).unwrap();
    
    // ファイルを変更
    fs::write(&test_file_path, "modified content").unwrap();
    
    // 変更されたことを確認
    let modified_content = fs::read_to_string(&test_file_path).unwrap();
    assert_eq!(modified_content, "modified content");
    
    // バックアップから復元
    backup::restore_backup(&backup_path, &test_file_path).unwrap();
    
    // 復元されたことを確認
    let restored_content = fs::read_to_string(&test_file_path).unwrap();
    assert_eq!(restored_content, "original content");
    
    // テスト環境のクリーンアップ
    common::cleanup_test_environment(&temp_dir);
} 