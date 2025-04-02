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
                                                        <v-text-field v-model="configUpdate.path.besiege_path"
                                                            label="Besiegeのパス" @update:model-value="updateBesiegePath"></v-text-field>
                                                    </v-col>
                                                    <v-col cols="12">
                                                        <v-text-field v-model="configUpdate.path.workshop_dir_path"
                                                            label="Steam Workshopのパス" @update:model-value="updateWorkshopPath"></v-text-field>
                                                    </v-col>
                                                    <v-col cols="12">
                                                        <v-text-field v-model="configUpdate.path.ballista_data_path"
                                                            label="Ballistaデータパス" @update:model-value="updateBallistaDataPath"></v-text-field>
                                                    </v-col>
                                                    <v-col cols="12" md="6">
                                                        <v-select v-model="configUpdate.ui.theme"
                                                            :items="['light', 'dark', 'system']" label="テーマ"
                                                            @update:model-value="updateUiTheme"></v-select>
                                                    </v-col>
                                                    <v-col cols="12" md="6">
                                                        <v-select v-model="configUpdate.ui.language"
                                                            :items="['ja', 'en']" label="言語"
                                                            @update:model-value="updateLanguage"></v-select>
                                                    </v-col>
                                                    <v-col cols="12">
                                                        <v-select v-model="configUpdate.logging.level"
                                                            :items="['trace', 'debug', 'info', 'warn', 'error', 'off']"
                                                            label="ログレベル"></v-select>
                                                    </v-col>
                                                </v-row>
                                            </v-card-text>
                                        </v-card>
                                    </v-col>
                                </v-row>
                            </v-card-text>
                        </v-window-item>

                        <!-- MOD管理API タブ -->
                        <v-window-item value="mods">
                            <v-card-text>
                                <v-alert type="info" variant="tonal">
                                    MOD管理API機能は現在利用できません
                                </v-alert>
                            </v-card-text>
                        </v-window-item>

                        <!-- バックアップAPI タブ -->
                        <v-window-item value="backup">
                            <v-card-text>
                                <v-alert type="info" variant="tonal">
                                    バックアップAPI機能は現在利用できません
                                </v-alert>
                            </v-card-text>
                        </v-window-item>

                        <!-- プリセットAPI タブ -->
                        <v-window-item value="preset">
                            <v-card-text>
                                <v-alert type="info" variant="tonal">
                                    プリセットAPI機能は現在利用できません
                                </v-alert>
                            </v-card-text>
                        </v-window-item>

                        <!-- ログタブ -->
                        <v-window-item value="logs">
                            <v-card-text>
                                <v-textarea v-model="logs" label="実行ログ" readonly variant="outlined" rows="20" auto-grow
                                    class="log-area" style="white-space: pre-wrap;"></v-textarea>
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
interface PathConfig {
    besiege_path: string;
    workshop_dir_path: string;
    ballista_data_path: string;
}

interface LoggingConfig {
    level: string;
    max_files: number;
}

interface UiConfig {
    theme: string;
    language: string;
}

interface AppConfig {
    path: PathConfig;
    logging: LoggingConfig;
    ui: UiConfig;
}

const configData = ref<AppConfig | null>(null);
const configDataString = computed(() => {
    return configData.value ? JSON.stringify(configData.value, null, 2) : '';
});

const configUpdate = ref<AppConfig>({
    path: {
        besiege_path: '',
        workshop_dir_path: '',
        ballista_data_path: ''
    },
    logging: {
        level: 'info',
        max_files: 10
    },
    ui: {
        theme: 'light',
        language: 'ja'
    }
});

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
 * Besiegeのパスを更新する
 */
async function updateBesiegePath(): Promise<void> {
    try {
        logMessage('info', `Besiegeのパスを更新中... (${configUpdate.value.path.besiege_path})`);
        await invoke('set_besiege_path', { path: configUpdate.value.path.besiege_path });
        logMessage('info', 'Besiegeのパスを更新しました');
        await getConfig(); // 設定を再取得して最新状態を反映
    } catch (error) {
        logMessage('error', `Besiegeパス更新エラー: ${error}`);
    }
}

/**
 * Steam Workshopのパスを更新する
 */
async function updateWorkshopPath(): Promise<void> {
    try {
        logMessage('info', `Steam Workshopのパスを更新中... (${configUpdate.value.path.workshop_dir_path})`);
        await invoke('set_workshop_path', { path: configUpdate.value.path.workshop_dir_path });
        logMessage('info', 'Steam Workshopのパスを更新しました');
        await getConfig(); // 設定を再取得して最新状態を反映
    } catch (error) {
        logMessage('error', `Workshopパス更新エラー: ${error}`);
    }
}

/**
 * Ballistaデータパスを更新する
 */
async function updateBallistaDataPath(): Promise<void> {
    try {
        logMessage('info', `Ballistaデータパスを更新中... (${configUpdate.value.path.ballista_data_path})`);
        await invoke('set_ballista_data_path', { path: configUpdate.value.path.ballista_data_path });
        logMessage('info', 'Ballistaデータパスを更新しました');
        await getConfig(); // 設定を再取得して最新状態を反映
    } catch (error) {
        logMessage('error', `Ballistaデータパス更新エラー: ${error}`);
    }
}

/**
 * UIテーマを更新する
 */
async function updateUiTheme(): Promise<void> {
    try {
        logMessage('info', `UIテーマを更新中... (${configUpdate.value.ui.theme})`);
        await invoke('set_ui_theme', { theme: configUpdate.value.ui.theme });
        logMessage('info', 'UIテーマを更新しました');
        await getConfig(); // 設定を再取得して最新状態を反映
    } catch (error) {
        logMessage('error', `UIテーマ更新エラー: ${error}`);
    }
}

/**
 * 言語設定を更新する
 */
async function updateLanguage(): Promise<void> {
    try {
        logMessage('info', `言語設定を更新中... (${configUpdate.value.ui.language})`);
        await invoke('set_language', { language: configUpdate.value.ui.language });
        logMessage('info', '言語設定を更新しました');
        await getConfig(); // 設定を再取得して最新状態を反映
    } catch (error) {
        logMessage('error', `言語設定更新エラー: ${error}`);
    }
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
