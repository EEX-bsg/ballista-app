/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! MOD管理モジュール
//!
//! このモジュールは、Besiegeのmodding.xmlファイルを解析・編集するための機能を提供します。
//! MODの有効/無効の切り替えやXMLの読み書きなどの操作が可能です。

use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read};
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesDecl};
use quick_xml::Reader;
use quick_xml::Writer;
use serde::{Deserialize, Serialize};
use log::{debug, info};

use crate::utils::validate_file_path;
use crate::error::BallistaError;
use crate::trace_fn;


/// MOD情報を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    /// MODの一意のID
    pub uuid: String,
    /// MODの名前
    pub name: String,
    /// MODのバージョン
    pub version: String,
    /// MODのソース（'W'=Workshop, 'L'=Local）
    pub source: String,
    /// SteamワークショップID（WorkshopのMODの場合のみ）
    pub workshop_id: Option<String>,
    /// MODが有効かどうか
    pub enabled: bool,
}

/// Modding.xmlのデータを表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModdingXmlData {
    /// 有効なMODのリスト
    pub enabled_mods: Vec<ModInfo>,
    /// 無効なMODのリスト
    pub disabled_mods: Vec<ModInfo>,
    /// ゲームのバージョン
    pub game_version: Option<String>
}

impl ModdingXmlData {
    /// XMLからJSONデータを生成する
    pub fn from_xml(xml_content: &str) -> Result<Self, BallistaError> {
        let xml_data = parse_modding_xml(xml_content)?;

        Ok(ModdingXmlData {
            enabled_mods: xml_data.enabled_mods,
            disabled_mods: xml_data.disabled_mods,
            game_version: xml_data.game_version,
        })
    }

    /// JSONからXMLデータを生成する
    pub fn to_xml(&self) -> Result<String, BallistaError> {
        let xml_data = ModdingXmlData {
            enabled_mods: self.enabled_mods.clone(),
            disabled_mods: self.disabled_mods.clone(),
            game_version: self.game_version.clone(),
        };

        generate_xml(&xml_data)
    }
}

/// Modding.xmlファイルを読み込む
pub fn read_modding_xml(file_path: &Path) -> Result<String, BallistaError> {
    trace_fn!("read_modding_xml(file_path: {})", file_path.display());
    
    // パス検証
    validate_file_path(file_path)?;
    
    // ファイルが存在することを確認
    if !file_path.exists() {
        return Err(BallistaError::FileError(
            format!("ファイルが存在しません: {}", file_path.display())
        ));
    }
    
    // ファイル内容を読み込み
    let mut content = String::new();
    let mut file = fs::File::open(file_path)
        .map_err(|e| BallistaError::FileError(format!("ファイルを開けませんでした: {}", e)))?;
    
    file.read_to_string(&mut content)
        .map_err(|e| BallistaError::FileError(format!("ファイルの読み込みに失敗しました: {}", e)))?;
    
    debug!("Modding.xmlファイルを読み込みました: {} バイト", content.len());
    Ok(content)
}

/// Modding.xmlファイルを読み込みJSONデータを返す
pub fn read_modding_xml_as_json(file_path: &Path) -> Result<ModdingXmlData, BallistaError> {
    let xml_content = read_modding_xml(file_path)?;
    ModdingXmlData::from_xml(&xml_content)
}

/// JSONデータをModding.xmlファイルに書き込む
pub fn write_modding_xml_from_json(file_path: &Path, data: &ModdingXmlData) -> Result<(), BallistaError> {
    let xml_content = data.to_xml()?;
    fs::write(file_path, xml_content)
        .map_err(|e| BallistaError::FileError(format!("ファイルの書き込みに失敗しました: {}", e)))
}

/// Modding.xmlファイルを解析する
pub fn parse_modding_xml(content: &str) -> Result<ModdingXmlData, BallistaError> {
    trace_fn!("parse_modding_xml(content: {} bytes)", content.len());
    
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut disabled_uuids = Vec::new();
    let mut game_version = None;
    
    // まず無効化されたモッドのUUIDを収集
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"StringArray" => {
                if let Some(key) = e.attributes()
                    .find(|attr| attr.as_ref().unwrap().key.as_ref() == b"key")
                    .map(|attr| attr.as_ref().unwrap().value.to_vec())
                {
                    if key == b"disabled-mods" {
                        let mut in_disabled_mods = true;
                        while in_disabled_mods {
                            match reader.read_event_into(&mut buf) {
                                Ok(Event::Start(ref e)) if e.name().as_ref() == b"String" => {
                                    if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                                        if let Ok(text) = e.unescape() {
                                            disabled_uuids.push(text.to_string());
                                        }
                                    }
                                }
                                Ok(Event::End(ref e)) if e.name().as_ref() == b"StringArray" => {
                                    in_disabled_mods = false;
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => return Err(BallistaError::XmlError(format!("XMLの解析エラー: {}", e))),
                                _ => (),
                            }
                        }
                    } else if key == b"maintenance-lastGameVersion" {
                        if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                            if let Ok(text) = e.unescape() {
                                game_version = Some(text.to_string());
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(BallistaError::XmlError(format!("XMLの解析エラー: {}", e))),
            _ => (),
        }
        buf.clear();
    }
    
    // 次にモッド情報を収集
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    buf.clear();
    
    let mut enabled_mods = Vec::new();
    let mut disabled_mods = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"StringArray" => {
                if let Some(key) = e.attributes()
                    .find(|attr| attr.as_ref().unwrap().key.as_ref() == b"key")
                    .map(|attr| attr.as_ref().unwrap().value.to_vec())
                {
                    if key == b"maintenance-lastMods" {
                        let mut in_mods = true;
                        while in_mods {
                            match reader.read_event_into(&mut buf) {
                                Ok(Event::Start(ref e)) if e.name().as_ref() == b"String" => {
                                    if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                                        if let Ok(text) = e.unescape() {
                                            let mod_info = text.to_string();
                                            let parts: Vec<&str> = mod_info.split('~').collect();
                                            if parts.len() >= 4 {
                                                let uuid = parts[0].to_string();
                                                let source = parts[1].to_string();
                                                let workshop_id = if source == "W" && parts.len() > 4 {
                                                    Some(parts[2].to_string())
                                                } else {
                                                    None
                                                };
                                                
                                                // バージョンと名前のインデックスを決定する
                                                let version_idx = parts.len() - 2;
                                                let name_idx = parts.len() - 1;
                                                
                                                let version = parts[version_idx].to_string();
                                                let name = parts[name_idx].to_string();
                                                
                                                let enabled = !disabled_uuids.contains(&uuid);
                                                
                                                let mod_info = ModInfo {
                                                    uuid,
                                                    name,
                                                    version,
                                                    source,
                                                    workshop_id,
                                                    enabled,
                                                };
                                                
                                                if enabled {
                                                    enabled_mods.push(mod_info);
                                                } else {
                                                    disabled_mods.push(mod_info);
                                                }
                                            }
                                        }
                                    }
                                }
                                Ok(Event::End(ref e)) if e.name().as_ref() == b"StringArray" => {
                                    in_mods = false;
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => return Err(BallistaError::XmlError(format!("XMLの解析エラー: {}", e))),
                                _ => (),
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(BallistaError::XmlError(format!("XMLの解析エラー: {}", e))),
            _ => (),
        }
        buf.clear();
    }
    
    info!("XMLの解析が完了しました: 有効なモッド {}, 無効なモッド {}",
            enabled_mods.len(), disabled_mods.len());
    
    Ok(ModdingXmlData {
        enabled_mods,
        disabled_mods,
        game_version
    })
}


/// XMLを生成する
///
/// ModdingXmlDataからModding.xmlの内容を生成します。
///
/// # 引数
///
/// * `data` - 生成元となるMODデータ
///
/// # 戻り値
///
/// 生成されたXML文字列、またはエラーメッセージ
pub fn generate_xml(data: &ModdingXmlData) -> Result<String, BallistaError> {
    trace_fn!("generate_xml(data)");
    
    let mut writer = Writer::new(Vec::new());
    
    // XML宣言を書き込む
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| BallistaError::XmlError(format!("XML宣言の書き込みに失敗しました: {}", e)))?;
    
    // ルート要素を開始
    let root = BytesStart::new("StringDict");
    writer.write_event(Event::Start(root))
        .map_err(|e| BallistaError::XmlError(format!("ルート要素の書き込みに失敗しました: {}", e)))?;
    
    // disabled-modsセクションを書き込む
    let mut disabled_elem = BytesStart::new("StringArray");
    disabled_elem.push_attribute(("key", "disabled-mods"));
    writer.write_event(Event::Start(disabled_elem))
        .map_err(|e| BallistaError::XmlError(format!("disabled-mods要素の書き込みに失敗しました: {}", e)))?;
    
    // 無効化されたMODのUUIDを書き込む
    for mod_info in &data.disabled_mods {
        let string_elem = BytesStart::new("String");
        writer.write_event(Event::Start(string_elem))
            .map_err(|e| BallistaError::XmlError(format!("String要素の書き込みに失敗しました: {}", e)))?;
        
        writer.write_event(Event::Text(quick_xml::events::BytesText::new(&mod_info.uuid)))
            .map_err(|e| BallistaError::XmlError(format!("MOD UUID書き込みに失敗しました: {}", e)))?;
        
        writer.write_event(Event::End(BytesEnd::new("String")))
            .map_err(|e| BallistaError::XmlError(format!("String要素終了の書き込みに失敗しました: {}", e)))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("StringArray")))
        .map_err(|e| BallistaError::XmlError(format!("StringArray要素終了の書き込みに失敗しました: {}", e)))?;
    
    // maintenance-lastModsセクションを書き込む
    let mut mods_elem = BytesStart::new("StringArray");
    mods_elem.push_attribute(("key", "maintenance-lastMods"));
    writer.write_event(Event::Start(mods_elem))
        .map_err(|e| BallistaError::XmlError(format!("maintenance-lastMods要素の書き込みに失敗しました: {}", e)))?;
    
    // すべてのMOD情報を書き込む
    let all_mods = [&data.enabled_mods[..], &data.disabled_mods[..]].concat();
    for mod_info in all_mods {
        let string_elem = BytesStart::new("String");
        writer.write_event(Event::Start(string_elem))
            .map_err(|e| BallistaError::XmlError(format!("String要素の書き込みに失敗しました: {}", e)))?;
        
        // MOD情報を~で区切って書き込む
        let mod_str = if mod_info.source == "W" && mod_info.workshop_id.is_some() {
            format!("{}~{}~{}~{}~{}", 
                mod_info.uuid, 
                mod_info.source, 
                mod_info.workshop_id.as_ref().unwrap(), 
                mod_info.version, 
                mod_info.name)
        } else {
            format!("{}~{}~~{}~{}", 
                mod_info.uuid, 
                mod_info.source, 
                mod_info.version, 
                mod_info.name)
        };
        
        writer.write_event(Event::Text(quick_xml::events::BytesText::new(&mod_str)))
            .map_err(|e| BallistaError::XmlError(format!("MOD情報書き込みに失敗しました: {}", e)))?;
        
        writer.write_event(Event::End(BytesEnd::new("String")))
            .map_err(|e| BallistaError::XmlError(format!("String要素終了の書き込みに失敗しました: {}", e)))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("StringArray")))
        .map_err(|e| BallistaError::XmlError(format!("StringArray要素終了の書き込みに失敗しました: {}", e)))?;
    
    // ゲームバージョンを書き込む（存在する場合）
    if let Some(version) = &data.game_version {
        let mut version_elem = BytesStart::new("String");
        version_elem.push_attribute(("key", "maintenance-lastGameVersion"));
        writer.write_event(Event::Start(version_elem))
            .map_err(|e| BallistaError::XmlError(format!("maintenance-lastGameVersion要素の書き込みに失敗しました: {}", e)))?;
        
        writer.write_event(Event::Text(quick_xml::events::BytesText::new(version)))
            .map_err(|e| BallistaError::XmlError(format!("ゲームバージョン書き込みに失敗しました: {}", e)))?;
        
        writer.write_event(Event::End(BytesEnd::new("String")))
            .map_err(|e| BallistaError::XmlError(format!("String要素終了の書き込みに失敗しました: {}", e)))?;
    }
    
    // ルート要素を閉じる
    writer.write_event(Event::End(BytesEnd::new("StringDict")))
        .map_err(|e| BallistaError::XmlError(format!("ルート要素終了の書き込みに失敗しました: {}", e)))?;
    
    // バイト配列を文字列に変換
    let result = String::from_utf8(writer.into_inner())
        .map_err(|e| BallistaError::XmlError(format!("XML文字列の生成に失敗しました: {}", e)))?;
    
    Ok(result)
}

/// MODの有効/無効状態を切り替える
///
/// 指定されたUUIDのMODの有効/無効状態を反転させます。
///
/// # 引数
///
/// * `data` - MODデータ（変更される）
/// * `uuid` - 切り替え対象のMODのUUID
///
/// # 戻り値
///
/// 成功した場合はOk、失敗した場合はエラーメッセージ
pub fn toggle_mod_enabled(data: &mut ModdingXmlData, uuid: &str) -> Result<(), BallistaError> {
    trace_fn!("toggle_mod_enabled(uuid: {})", uuid);
    
    // 有効なMODリストから探す
    if let Some(pos) = data.enabled_mods.iter().position(|m| m.uuid == uuid) {
        let mut mod_info = data.enabled_mods.remove(pos);
        mod_info.enabled = false;
        data.disabled_mods.push(mod_info);
        return Ok(());
    }
    
    // 無効なMODリストから探す
    if let Some(pos) = data.disabled_mods.iter().position(|m| m.uuid == uuid) {
        let mut mod_info = data.disabled_mods.remove(pos);
        mod_info.enabled = true;
        data.enabled_mods.push(mod_info);
        return Ok(());
    }
    
    Err(BallistaError::PresetError(format!("指定されたUUIDのMODが見つかりません: {}", uuid)))
}

/// MOD情報を取得する
///
/// 指定されたUUIDのMOD情報を検索します。
///
/// # 引数
///
/// * `data` - MODデータ
/// * `uuid` - 検索するMODのUUID
///
/// # 戻り値
///
/// MOD情報（見つかった場合）またはNone（見つからなかった場合）
pub fn get_mod_info(data: &ModdingXmlData, uuid: &str) -> Option<ModInfo> {
    trace_fn!("get_mod_info(uuid: {})", uuid);
    
    // 有効なMODリストから探す
    if let Some(mod_info) = data.enabled_mods.iter().find(|m| m.uuid == uuid) {
        return Some(mod_info.clone());
    }
    
    // 無効なMODリストから探す
    if let Some(mod_info) = data.disabled_mods.iter().find(|m| m.uuid == uuid) {
        return Some(mod_info.clone());
    }
    
    None
}
