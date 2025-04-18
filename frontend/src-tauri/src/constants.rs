/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Constants Settings
//!
//! このファイルではアプリで不変の設定(定数)を保持する


/// Besiege.exeのパスから各ファイル・ディレクトリを取得する事を目的としたパス郡
/// Modsディレクトリのパス(Besiegeディレクトリからの相対座標)
pub const MODS_DIR_RELATIVE_PATH: &str = "Besiege_Data/Mods";

/// Modding.xmlのパス (Besiegeディレクトリからの相対座標)
pub const MODDING_CONFIG_RELATIVE_PATH: &str = "Besiege_Data/Mods/Config/Modding.xml";

/// BesiegeWorkshopディレクトリのパス(workshopディレクトリのパスからの相対座標)
pub const BESIEGE_WORKSHOP_DIR_RELATIVE_PATH: &str = "content/346010";

/// デフォルト設定用の定数
/// デフォルトのBesiege.exeのパス
pub const DEFAULT_BESIEGE_PATH: &str = "C:/Program Files (x86)/Steam/steamapps/common/Besiege/Besiege.exe";

/// デフォルトのSteam Workshopのディレクトリパス
pub const DEFAULT_WORKSHOP_DIR_PATH: &str = "C:/Program Files (x86)/Steam/steamapps/workshop/";

/// デフォルトのBallistaアプリデータディレクトリパス（環境変数展開前）
pub const DEFAULT_BALLISTA_DATA_PATH: &str = "$HOME/AppData/Roaming/ballista-app/";

/// デフォルトのログレベル
pub const DEFAULT_LOG_LEVEL: &str = "trace";

/// デフォルトの最大ログファイル数
pub const DEFAULT_MAX_LOG_FILES: usize = 10;

/// デフォルトのUIテーマ
pub const DEFAULT_UI_THEME: &str = "light";

/// デフォルトの言語設定
pub const DEFAULT_LANGUAGE: &str = "ja";
