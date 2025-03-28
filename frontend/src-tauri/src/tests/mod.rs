/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! テストモジュール

pub mod common;
pub mod modding_tests;
pub mod backup_tests;
pub mod preset_tests;
pub mod api_tests; 

// 以前のテストを新しいモジュールに移行させるための再エクスポート
pub use modding_tests::*;
pub use backup_tests::*;
pub use preset_tests::*;
pub use api_tests::*; 