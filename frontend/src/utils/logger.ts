/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { ref, Ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

/**
 * ロガークラス - フロントエンドのログ管理を担当
 */
export class Logger {
    private logsRef: Ref<string>;

    /**
     * ロガーを初期化
     * @param logsRef - ログを保存するリアクティブな参照
     */
    constructor(logsRef: Ref<string>) {
        this.logsRef = logsRef;
    }

    /**
     * ログメッセージを記録および表示する
     * フロントエンドのログエリアに出力し、コンソールにも出力し、Rustバックエンドにも送信する
     * 
     * @param level - ログレベル ('info', 'warn', 'error', 'debug')
     * @param message - ログメッセージ本文
     */
    public log(level: string, message: string): void {
        const now = new Date().toLocaleString();
        this.logsRef.value += `[${now}] [${level.toUpperCase()}] ${message}\n`;

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
     * 情報レベルのログを記録
     * @param message - ログメッセージ
     */
    public info(message: string): void {
        this.log('info', message);
    }

    /**
     * 警告レベルのログを記録
     * @param message - ログメッセージ
     */
    public warn(message: string): void {
        this.log('warn', message);
    }

    /**
     * エラーレベルのログを記録
     * @param message - ログメッセージ
     */
    public error(message: string): void {
        this.log('error', message);
    }

    /**
     * デバッグレベルのログを記録
     * @param message - ログメッセージ
     */
    public debug(message: string): void {
        this.log('debug', message);
    }

    /**
     * ログエリアをクリアする
     */
    public clear(): void {
        this.logsRef.value = '';
    }
}

/**
 * ロガーインスタンスを作成する
 * @param logsRef - ログを保存するリアクティブな参照
 * @returns ロガーインスタンス
 */
export function createLogger(logsRef: Ref<string>): Logger {
    return new Logger(logsRef);
}
