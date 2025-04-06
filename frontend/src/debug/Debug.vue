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
                    <v-window v-model="activeTab" :show-arrows="false" :touch="false">
                        <!-- 設定系API タブ -->
                        <v-window-item value="config">
                            <config-tab :log-message="logMessage" @config-updated="onConfigUpdated" />
                        </v-window-item>

                        <!-- MOD管理API タブ -->
                        <v-window-item value="mods">
                            <mods-tab />
                        </v-window-item>

                        <!-- バックアップAPI タブ -->
                        <v-window-item value="backup">
                            <backup-tab />
                        </v-window-item>

                        <!-- プリセットAPI タブ -->
                        <v-window-item value="preset">
                            <preset-tab />
                        </v-window-item>

                        <!-- ログタブ -->
                        <v-window-item value="logs">
                            <logs-tab :logs="logs" />
                        </v-window-item>
                    </v-window>
                </v-card>
            </v-container>
        </v-main>
    </v-app>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { createLogger } from '../utils/logger';
import ConfigTab from './components/ConfigTab.vue';
import ModsTab from './components/ModsTab.vue';
import BackupTab from './components/BackupTab.vue';
import PresetTab from './components/PresetTab.vue';
import LogsTab from './components/LogsTab.vue';

// 状態管理
const activeTab = ref('config');
const logs = ref('');
const logger = createLogger(logs);

/**
 * ログメッセージを記録および表示する
 * @param level - ログレベル ('info', 'warn', 'error', 'debug')
 * @param message - ログメッセージ本文
 */
function logMessage(level: string, message: string): void {
    logger.log(level, message);
}

/**
 * ログエリアをクリアする
 */
function clearLogs(): void {
    logger.clear();
}

/**
 * 設定が更新されたときのイベントハンドラ
 */
function onConfigUpdated(): void {
    logMessage('info', '設定が更新されました');
}

// 初期化
onMounted(() => {
    logger.info('APIテスト環境を初期化しています...');
    
    // 初期化時にログが表示されるかテスト
    logger.info('ログテスト: これはAPIテスト環境の初期化時のログです');
    logger.warn('ログテスト: これは警告レベルのログです');
    logger.error('ログテスト: これはエラーレベルのログです');
    
    logger.info('APIテスト環境の初期化が完了しました');
});
</script>
