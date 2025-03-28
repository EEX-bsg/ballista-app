/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<template>
    <v-app>
        <v-app-bar color="primary" density="compact">
            <v-app-bar-title class="text-h6 font-weight-bold">
                Ballista - API テスト環境
            </v-app-bar-title>
            <v-spacer></v-spacer>
            <v-btn @click="clearLogs" color="error" variant="text">
                ログをクリア
            </v-btn>
        </v-app-bar>

        <v-main class="bg-grey-lighten-4">
            <v-container fluid>
                <!-- API テストタブ -->
                <v-tabs v-model="activeTab" bg-color="primary" color="white">
                    <v-tab value="config">設定系API</v-tab>
                    <v-tab value="mods">MOD管理API</v-tab>
                    <v-tab value="backup">バックアップAPI</v-tab>
                    <v-tab value="preset">プリセットAPI</v-tab>
                    <v-tab value="logs">ログ</v-tab>
                </v-tabs>

                <v-card>
                    <v-window v-model="activeTab">
                        <!-- 設定系API タブ -->
                        <v-window-item value="config">
                            <v-card-text>
                                <v-row>
                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined">
                                            <v-card-title class="bg-blue-lighten-5">
                                                設定取得
                                            </v-card-title>
                                            <v-card-text>
                                                <v-btn color="primary" block @click="getConfig" class="mb-4">
                                                    設定を取得
                                                </v-btn>
                                                <v-textarea v-if="configData" v-model="configDataString" label="現在の設定"
                                                    readonly rows="10" auto-grow></v-textarea>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>
                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined">
                                            <v-card-title class="bg-blue-lighten-5">
                                                設定更新
                                            </v-card-title>
                                            <v-card-text>
                                                <v-row>
                                                    <v-col cols="12">
                                                        <v-select v-model="configUpdate.logging.level"
                                                            :items="['trace', 'debug', 'info', 'warn', 'error']"
                                                            label="ログレベル"></v-select>
                                                    </v-col>
                                                    <v-col cols="12" md="6">
                                                        <v-text-field
                                                            v-model.number="configUpdate.logging.file_rotation.max_files"
                                                            label="最大ファイル数" type="number"></v-text-field>
                                                    </v-col>
                                                    <v-col cols="12" md="6">
                                                        <v-text-field
                                                            v-model.number="configUpdate.logging.file_rotation.max_size_mb"
                                                            label="最大サイズ(MB)" type="number"></v-text-field>
                                                    </v-col>
                                                    <v-col cols="12" md="6">
                                                        <v-text-field v-model.number="configUpdate.modding.backup_count"
                                                            label="バックアップ数" type="number"></v-text-field>
                                                    </v-col>
                                                    <v-col cols="12" md="6">
                                                        <v-switch v-model="configUpdate.modding.auto_backup"
                                                            label="自動バックアップ" color="primary"></v-switch>
                                                    </v-col>
                                                    <v-col cols="12" md="6">
                                                        <v-select v-model="configUpdate.ui.theme"
                                                            :items="['light', 'dark', 'system']" label="テーマ"></v-select>
                                                    </v-col>
                                                    <v-col cols="12" md="6">
                                                        <v-select v-model="configUpdate.ui.language"
                                                            :items="['ja', 'en']" label="言語"></v-select>
                                                    </v-col>
                                                </v-row>
                                                <v-btn color="success" block @click="updateConfig">
                                                    設定を更新
                                                </v-btn>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>
                                </v-row>
                            </v-card-text>
                        </v-window-item>

                        <!-- MOD管理API タブ -->
                        <v-window-item value="mods">
                            <v-card-text>
                                <v-row>
                                    <v-col cols="12">
                                        <v-btn color="primary" class="mr-2" @click="getMods">
                                            MOD一覧を取得
                                        </v-btn>
                                        <v-btn color="info" @click="refreshMods">
                                            MOD一覧を更新
                                        </v-btn>
                                    </v-col>
                                </v-row>

                                <v-row>
                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined" class="mb-4">
                                            <v-card-title class="bg-green-lighten-4">
                                                MODの有効化
                                            </v-card-title>
                                            <v-card-text>
                                                <v-text-field v-model="modUuid" label="MOD UUID"
                                                    placeholder="有効化するMODのUUIDを入力" variant="outlined"></v-text-field>
                                                <v-btn color="success" block @click="enableMod" :disabled="!modUuid">
                                                    MODを有効化
                                                </v-btn>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>

                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined" class="mb-4">
                                            <v-card-title class="bg-red-lighten-4">
                                                MODの無効化
                                            </v-card-title>
                                            <v-card-text>
                                                <v-text-field v-model="modUuid" label="MOD UUID"
                                                    placeholder="無効化するMODのUUIDを入力" variant="outlined"></v-text-field>
                                                <v-btn color="error" block @click="disableMod" :disabled="!modUuid">
                                                    MODを無効化
                                                </v-btn>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>
                                </v-row>

                                <v-card variant="outlined">
                                    <v-card-title class="bg-blue-lighten-5">
                                        MOD一覧
                                    </v-card-title>
                                    <v-card-text v-if="mods.length > 0">
                                        <v-data-table :headers="modHeaders" :items="mods" item-value="uuid"
                                            density="compact">
                                            <template v-slot:item.source="{ item }">
                                                <v-chip size="small" :color="item.source === 'W' ? 'blue' : 'grey'">
                                                    {{ item.source === 'W' ? 'Workshop' : 'Local' }}
                                                </v-chip>
                                            </template>
                                            <template v-slot:item.actions="{ item }">
                                                <v-btn size="small" color="success" variant="text" icon
                                                    @click="enableMod(item.uuid)" title="有効化">
                                                    <v-icon>mdi-check</v-icon>
                                                </v-btn>
                                                <v-btn size="small" color="error" variant="text" icon
                                                    @click="disableMod(item.uuid)" title="無効化">
                                                    <v-icon>mdi-close</v-icon>
                                                </v-btn>
                                            </template>
                                        </v-data-table>
                                    </v-card-text>
                                    <v-card-text v-else>
                                        MODが読み込まれていません
                                    </v-card-text>
                                </v-card>
                            </v-card-text>
                        </v-window-item>

                        <!-- バックアップAPI タブ -->
                        <v-window-item value="backup">
                            <v-card-text>
                                <v-row>
                                    <v-col cols="12">
                                        <v-btn color="primary" class="mr-2" @click="getBackups">
                                            バックアップ一覧を取得
                                        </v-btn>
                                    </v-col>
                                </v-row>

                                <v-row>
                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined" class="mb-4">
                                            <v-card-title class="bg-green-lighten-4">
                                                バックアップ作成
                                            </v-card-title>
                                            <v-card-text>
                                                <v-text-field v-model="backupName" label="バックアップ名"
                                                    placeholder="バックアップの名前を入力" variant="outlined"></v-text-field>
                                                <v-btn color="success" block @click="createBackup">
                                                    バックアップを作成
                                                </v-btn>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>

                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined" class="mb-4">
                                            <v-card-title class="bg-blue-lighten-4">
                                                バックアップ復元
                                            </v-card-title>
                                            <v-card-text>
                                                <v-text-field v-model="backupId" label="バックアップID"
                                                    placeholder="復元するバックアップのIDを入力" variant="outlined"></v-text-field>
                                                <v-btn color="info" block @click="restoreBackup" :disabled="!backupId">
                                                    バックアップを復元
                                                </v-btn>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>
                                </v-row>

                                <v-card variant="outlined">
                                    <v-card-title class="bg-purple-lighten-5">
                                        バックアップ削除
                                    </v-card-title>
                                    <v-card-text>
                                        <v-text-field v-model="backupId" label="バックアップID" placeholder="削除するバックアップのIDを入力"
                                            variant="outlined"></v-text-field>
                                        <v-btn color="error" block @click="deleteBackup" :disabled="!backupId">
                                            バックアップを削除
                                        </v-btn>
                                    </v-card-text>
                                </v-card>

                                <v-card variant="outlined" class="mt-4">
                                    <v-card-title class="bg-blue-lighten-5">
                                        バックアップ一覧
                                    </v-card-title>
                                    <v-card-text v-if="backups.length > 0">
                                        <v-data-table :headers="backupHeaders" :items="backups" item-value="id"
                                            density="compact">
                                            <template v-slot:item.date="{ item }">
                                                {{ new Date(item.date).toLocaleString() }}
                                            </template>
                                            <template v-slot:item.actions="{ item }">
                                                <v-btn size="small" color="info" variant="text" icon
                                                    @click="restoreBackup(item.id)" title="復元">
                                                    <v-icon>mdi-restore</v-icon>
                                                </v-btn>
                                                <v-btn size="small" color="error" variant="text" icon
                                                    @click="deleteBackup(item.id)" title="削除">
                                                    <v-icon>mdi-delete</v-icon>
                                                </v-btn>
                                            </template>
                                        </v-data-table>
                                    </v-card-text>
                                    <v-card-text v-else>
                                        バックアップが存在しません
                                    </v-card-text>
                                </v-card>
                            </v-card-text>
                        </v-window-item>

                        <!-- プリセットAPI タブ -->
                        <v-window-item value="preset">
                            <v-card-text>
                                <v-row>
                                    <v-col cols="12">
                                        <v-btn color="primary" class="mr-2" @click="getPresets">
                                            プリセット一覧を取得
                                        </v-btn>
                                    </v-col>
                                </v-row>

                                <v-row>
                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined" class="mb-4">
                                            <v-card-title class="bg-green-lighten-4">
                                                プリセット作成
                                            </v-card-title>
                                            <v-card-text>
                                                <v-text-field v-model="presetName" label="プリセット名"
                                                    placeholder="プリセットの名前を入力" variant="outlined"
                                                    class="mb-2"></v-text-field>
                                                <v-textarea v-model="presetDescription" label="説明"
                                                    placeholder="プリセットの説明を入力" variant="outlined" rows="3"
                                                    class="mb-2"></v-textarea>
                                                <v-btn color="success" block @click="createPreset"
                                                    :disabled="!presetName">
                                                    プリセットを作成
                                                </v-btn>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>

                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined" class="mb-4">
                                            <v-card-title class="bg-blue-lighten-4">
                                                プリセット適用
                                            </v-card-title>
                                            <v-card-text>
                                                <v-text-field v-model="presetId" label="プリセットID"
                                                    placeholder="適用するプリセットのIDを入力" variant="outlined"></v-text-field>
                                                <v-btn color="info" block @click="applyPreset" :disabled="!presetId">
                                                    プリセットを適用
                                                </v-btn>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>
                                </v-row>

                                <v-row>
                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined" class="mb-4">
                                            <v-card-title class="bg-purple-lighten-4">
                                                プリセット更新
                                            </v-card-title>
                                            <v-card-text>
                                                <v-text-field v-model="presetId" label="プリセットID"
                                                    placeholder="更新するプリセットのIDを入力" variant="outlined"
                                                    class="mb-2"></v-text-field>
                                                <v-text-field v-model="presetUpdateName" label="新しい名前"
                                                    placeholder="新しいプリセット名（変更しない場合は空欄）" variant="outlined"
                                                    class="mb-2"></v-text-field>
                                                <v-textarea v-model="presetUpdateDescription" label="新しい説明"
                                                    placeholder="新しい説明（変更しない場合は空欄）" variant="outlined" rows="3"
                                                    class="mb-2"></v-textarea>
                                                <v-btn color="warning" block @click="updatePreset"
                                                    :disabled="!presetId">
                                                    プリセットを更新
                                                </v-btn>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>

                                    <v-col cols="12" md="6">
                                        <v-card variant="outlined" class="mb-4">
                                            <v-card-title class="bg-red-lighten-4">
                                                プリセット削除
                                            </v-card-title>
                                            <v-card-text>
                                                <v-text-field v-model="presetId" label="プリセットID"
                                                    placeholder="削除するプリセットのIDを入力" variant="outlined"></v-text-field>
                                                <v-btn color="error" block @click="deletePreset" :disabled="!presetId">
                                                    プリセットを削除
                                                </v-btn>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>
                                </v-row>

                                <v-card variant="outlined">
                                    <v-card-title class="bg-blue-lighten-5">
                                        プリセット一覧
                                    </v-card-title>
                                    <v-card-text v-if="presets.length > 0">
                                        <v-data-table :headers="presetHeaders" :items="presets" item-value="id"
                                            density="compact">
                                            <template v-slot:item.date="{ item }">
                                                {{ new Date(item.date).toLocaleString() }}
                                            </template>
                                            <template v-slot:item.actions="{ item }">
                                                <v-btn size="small" color="info" variant="text" icon
                                                    @click="applyPreset(item.id)" title="適用">
                                                    <v-icon>mdi-check</v-icon>
                                                </v-btn>
                                                <v-btn size="small" color="warning" variant="text" icon
                                                    @click="fillUpdateForm(item)" title="更新">
                                                    <v-icon>mdi-pencil</v-icon>
                                                </v-btn>
                                                <v-btn size="small" color="error" variant="text" icon
                                                    @click="deletePreset(item.id)" title="削除">
                                                    <v-icon>mdi-delete</v-icon>
                                                </v-btn>
                                            </template>
                                        </v-data-table>
                                    </v-card-text>
                                    <v-card-text v-else>
                                        プリセットが存在しません
                                    </v-card-text>
                                </v-card>
                            </v-card-text>
                        </v-window-item>

                        <!-- ログタブ -->
                        <v-window-item value="logs">
                            <v-card-text>
                                <v-textarea v-model="logs" label="実行ログ" readonly variant="outlined" rows="20" auto-grow
                                    class="log-area"></v-textarea>
                            </v-card-text>
                        </v-window-item>
                    </v-window>
                </v-card>
            </v-container>
        </v-main>
    </v-app>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

// 状態管理
const activeTab = ref('config');
const logs = ref('');

// 設定関連
interface FileRotationConfig {
    max_files: number;
    max_size_mb: number;
}

interface LoggingConfig {
    level: string;
    file_rotation: FileRotationConfig;
}

interface ModdingConfig {
    backup_count: number;
    auto_backup: boolean;
}

interface UiConfig {
    theme: string;
    language: string;
}

interface AppConfig {
    logging: LoggingConfig;
    modding: ModdingConfig;
    ui: UiConfig;
}

const configData = ref<AppConfig | null>(null);
const configDataString = computed(() => {
    return configData.value ? JSON.stringify(configData.value, null, 2) : '';
});

const configUpdate = ref<AppConfig>({
    logging: {
        level: 'info',
        file_rotation: {
            max_files: 10,
            max_size_mb: 10
        }
    },
    modding: {
        backup_count: 10,
        auto_backup: true
    },
    ui: {
        theme: 'light',
        language: 'ja'
    }
});

// MOD関連
interface ModInfo {
    uuid: string;
    name: string;
    source: string;
    version: string;
    workshop_id?: string;
    enabled?: boolean;
}

interface ModdingXmlData {
    enabled_mods: ModInfo[];
    disabled_mods: ModInfo[];
    path?: string;
}

const mods = ref<ModInfo[]>([]);
const modUuid = ref('');

const modHeaders = [
    { title: '名前', key: 'name' },
    { title: '種類', key: 'source' },
    { title: 'バージョン', key: 'version' },
    { title: 'Workshop ID', key: 'workshop_id' },
    { title: 'UUID', key: 'uuid' },
    { title: 'アクション', key: 'actions' }
];

// バックアップ関連
interface BackupInfo {
    id: string;
    name: string;
    date: string;
    size: number;
}

const backups = ref<BackupInfo[]>([]);
const backupName = ref('');
const backupId = ref('');

const backupHeaders = [
    { title: 'ID', key: 'id' },
    { title: '名前', key: 'name' },
    { title: '日時', key: 'date' },
    { title: 'サイズ', key: 'size' },
    { title: 'アクション', key: 'actions' }
];

// プリセット関連
interface PresetInfo {
    id: string;
    name: string;
    description: string;
    date: string;
}

const presets = ref<PresetInfo[]>([]);
const presetName = ref('');
const presetDescription = ref('');
const presetId = ref('');
const presetUpdateName = ref('');
const presetUpdateDescription = ref('');

const presetHeaders = [
    { title: 'ID', key: 'id' },
    { title: '名前', key: 'name' },
    { title: '説明', key: 'description' },
    { title: '日時', key: 'date' },
    { title: 'アクション', key: 'actions' }
];

/**
 * ログメッセージを記録および表示する
 * フロントエンドのログエリアに出力し、コンソールにも出力し、Rustバックエンドにも送信する
 * 
 * @param level - ログレベル ('info', 'warn', 'error', 'debug')
 * @param message - ログメッセージ本文
 */
function logMessage(level: string, message: string): void {
    const now = new Date().toLocaleString();
    logs.value += `[${now}] [${level.toUpperCase()}] ${message}\n`;

    // コンソールにも出力
    switch (level) {
        case 'error':
            console.error(message);
            break;
        case 'warn':
            console.warn(message);
            break;
        case 'info':
            console.info(message);
            break;
        case 'debug':
            console.debug(message);
            break;
        default:
            console.log(message);
    }

    // Rustのロガーにも送信
    invoke('log_message', { level, message: `[Frontend] ${message}` }).catch(err => {
        console.error('ログ送信エラー:', err);
    });
}

/**
 * ログエリアをクリアする
 */
function clearLogs(): void {
    logs.value = '';
}

// 設定系API
/**
 * バックエンドから設定を取得する
 * 取得した設定はconfigDataとconfigUpdateに保存される
 */
async function getConfig(): Promise<void> {
    try {
        logMessage('info', '設定を取得中...');
        const result = await invoke<AppConfig>('get_config');
        configData.value = result;
        configUpdate.value = JSON.parse(JSON.stringify(result)); // deep copy
        logMessage('info', '設定を取得しました');
    } catch (error) {
        logMessage('error', `設定取得エラー: ${error}`);
    }
}

/**
 * 現在のconfigUpdate状態をバックエンドに送信して設定を更新する
 */
async function updateConfig(): Promise<void> {
    try {
        logMessage('info', '設定を更新中...');
        const result = await invoke<AppConfig>('set_log_level_command', { config: configUpdate.value });
        configData.value = result;
        logMessage('info', '設定を更新しました');
    } catch (error) {
        logMessage('error', `設定更新エラー: ${error}`);
    }
}

// MOD管理API
/**
 * バックエンドからMOD一覧を取得する
 * 有効化されたMODと無効化されたMODを結合して表示する
 */
async function getMods(): Promise<void> {
    try {
        logMessage('info', 'MOD一覧を取得中...');
        const result = await invoke<ModdingXmlData>('load_modding_xml');
        if (result && result.enabled_mods && result.disabled_mods) {
            mods.value = [...result.enabled_mods, ...result.disabled_mods];
            logMessage('info', `MOD一覧を取得しました (${mods.value.length}件)`);
        } else {
            logMessage('warn', 'MOD一覧の取得結果が期待と異なります');
        }
    } catch (error) {
        logMessage('error', `MOD一覧取得エラー: ${error}`);
    }
}

/**
 * 指定されたUUIDのMODを有効化する
 *
 * @param uuid - 有効化するMODのUUID（省略時はmodUuid.valueを使用）
 */
async function enableMod(uuid?: string): Promise<void> {
    const targetUuid = uuid || modUuid.value;
    if (!targetUuid) return;

    try {
        logMessage('info', `MODを有効化中... (UUID: ${targetUuid})`);
        await invoke('enable_mod', { uuid: targetUuid });
        logMessage('info', 'MODを有効化しました');
        await getMods(); // 一覧を更新
    } catch (error) {
        logMessage('error', `MOD有効化エラー: ${error}`);
    }
}

/**
 * 指定されたUUIDのMODを無効化する
 *
 * @param uuid - 無効化するMODのUUID（省略時はmodUuid.valueを使用）
 */
async function disableMod(uuid?: string): Promise<void> {
    const targetUuid = uuid || modUuid.value;
    if (!targetUuid) return;

    try {
        logMessage('info', `MODを無効化中... (UUID: ${targetUuid})`);
        await invoke('disable_mod', { uuid: targetUuid });
        logMessage('info', 'MODを無効化しました');
        await getMods(); // 一覧を更新
    } catch (error) {
        logMessage('error', `MOD無効化エラー: ${error}`);
    }
}

/**
 * MOD一覧を再取得して更新する
 */
async function refreshMods(): Promise<void> {
    try {
        logMessage('info', 'MOD一覧を更新中...');
        await getMods();
        logMessage('info', `MOD一覧を更新しました (${mods.value.length}件)`);
    } catch (error) {
        logMessage('error', `MOD一覧更新エラー: ${error}`);
    }
}

// バックアップAPI
/**
 * バックアップ一覧を取得する
 * ユーザーにファイル選択ダイアログを表示してModding.xmlを選択してもらう
 */
async function getBackups(): Promise<void> {
    try {
        logMessage('info', 'バックアップ一覧を取得中...');
        // ファイルパスの取得が必要
        const filePath = await invoke<string>('select_modding_xml');
        const result = await invoke<BackupInfo[]>('get_backup_list', { file_path: filePath });
        backups.value = result;
        logMessage('info', `バックアップ一覧を取得しました (${result.length}件)`);
    } catch (error) {
        logMessage('error', `バックアップ一覧取得エラー: ${error}`);
    }
}

/**
 * 現在のModding.xmlのバックアップを作成する
 * ユーザーにファイル選択ダイアログを表示してModding.xmlを選択してもらう
 */
async function createBackup(): Promise<void> {
    try {
        logMessage('info', 'バックアップを作成中...');
        // ファイルパスの取得が必要
        const filePath = await invoke<string>('select_modding_xml');
        const backupPath = await invoke<string>('create_backup_file', { file_path: filePath });
        logMessage('info', `バックアップを作成しました (パス: ${backupPath})`);
        await getBackups(); // 一覧を更新
        backupName.value = '';
    } catch (error) {
        logMessage('error', `バックアップ作成エラー: ${error}`);
    }
}

async function restoreBackup(id?: string): Promise<void> {
    const targetId = id || backupId.value;
    if (!targetId) return;

    try {
        logMessage('info', `バックアップを復元中... (ID: ${targetId})`);
        // 元のファイルパスとバックアップパスの両方が必要
        const targetPath = await invoke<string>('select_modding_xml');
        await invoke('restore_backup', {
            backup_path: targetId,
            target_path: targetPath
        });
        logMessage('info', 'バックアップを復元しました');
    } catch (error) {
        logMessage('error', `バックアップ復元エラー: ${error}`);
    }
}

async function deleteBackup(id?: string): Promise<void> {
    const targetId = id || backupId.value;
    if (!targetId) return;

    try {
        logMessage('info', `バックアップを削除中... (ID: ${targetId})`);
        // このAPIは実装されていないようなので、機能実装までは無効化
        // await invoke('delete_backup', { id: targetId });
        logMessage('warn', 'バックアップ削除機能は未実装です');
        // await getBackups(); // 一覧を更新
    } catch (error) {
        logMessage('error', `バックアップ削除エラー: ${error}`);
    }
}

// プリセットAPI
/**
 * バックエンドからプリセット一覧を取得する
 */
async function getPresets(): Promise<void> {
    try {
        logMessage('info', 'プリセット一覧を取得中...');
        const result = await invoke<PresetInfo[]>('get_preset_list');
        presets.value = result;
        logMessage('info', `プリセット一覧を取得しました (${result.length}件)`);
    } catch (error) {
        logMessage('error', `プリセット一覧取得エラー: ${error}`);
    }
}

/**
 * 新しいプリセットを作成する
 */
async function createPreset(): Promise<void> {
    if (!presetName.value) return;

    try {
        logMessage('info', 'プリセットを作成中...');
        // XMLデータの取得が必要
        const xmlData = await invoke<ModdingXmlData>('load_modding_xml');
        const description = presetDescription.value || undefined;
        const result = await invoke<string>('save_preset', {
            name: presetName.value,
            description,
            data: xmlData
        });
        logMessage('info', `プリセットを作成しました (パス: ${result})`);
        await getPresets(); // 一覧を更新
        presetName.value = '';
        presetDescription.value = '';
    } catch (error) {
        logMessage('error', `プリセット作成エラー: ${error}`);
    }
}

/**
 * 指定されたプリセットを適用する
 *
 * @param id - 適用するプリセットのID（省略時はpresetId.valueを使用）
 */
async function applyPreset(id?: string): Promise<void> {
    const targetId = id || presetId.value;
    if (!targetId) return;

    try {
        logMessage('info', `プリセットを適用中... (ID: ${targetId})`);
        // ターゲットパスの取得が必要
        const targetPath = await invoke<string>('select_modding_xml');
        await invoke('apply_preset', {
            preset_path: targetId,
            target_path: targetPath
        });
        logMessage('info', 'プリセットを適用しました');
    } catch (error) {
        logMessage('error', `プリセット適用エラー: ${error}`);
    }
}

/**
 * 指定されたプリセットを削除する
 *
 * @param id - 削除するプリセットのID（省略時はpresetId.valueを使用）
 */
async function deletePreset(id?: string): Promise<void> {
    const targetId = id || presetId.value;
    if (!targetId) return;

    try {
        logMessage('info', `プリセットを削除中... (ID: ${targetId})`);
        await invoke('delete_preset', { preset_path: targetId });
        logMessage('info', 'プリセットを削除しました');
        await getPresets(); // 一覧を更新
    } catch (error) {
        logMessage('error', `プリセット削除エラー: ${error}`);
    }
}

/**
 * 指定されたプリセットを更新する
 */
async function updatePreset(): Promise<void> {
    if (!presetId.value) return;

    try {
        logMessage('info', `プリセットを更新中... (ID: ${presetId.value})`);
        // このAPIは実装されていないようなので、機能実装までは無効化
        logMessage('warn', 'プリセット更新機能は未実装です');

        /*
        const name = presetUpdateName.value || undefined;
        const description = presetUpdateDescription.value || undefined;
        
        const result = await invoke<PresetInfo>('update_preset', { 
            id: presetId.value, 
            name, 
            description 
        });
        
        logMessage('info', `プリセットを更新しました (ID: ${result.id})`);
        await getPresets(); // 一覧を更新
        
        // フォームをクリア
        presetUpdateName.value = '';
        presetUpdateDescription.value = '';
        */
    } catch (error) {
        logMessage('error', `プリセット更新エラー: ${error}`);
    }
}

function fillUpdateForm(preset: PresetInfo): void {
    presetId.value = preset.id;
    presetUpdateName.value = preset.name;
    presetUpdateDescription.value = preset.description;
    activeTab.value = 'preset'; // プリセットタブに移動
}

// 初期化
onMounted(async () => {
    logMessage('info', 'APIテスト環境を初期化しています...');
    try {
        await getConfig();
        logMessage('info', 'APIテスト環境の初期化が完了しました');
    } catch (error) {
        logMessage('error', `初期化エラー: ${error}`);
    }
});
</script>

<style>
.log-area {
    font-family: monospace;
    white-space: pre;
    font-size: 0.8rem;
}
</style>