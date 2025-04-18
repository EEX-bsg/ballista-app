/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import { createApp } from 'vue'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import 'vuetify/styles'
import '@mdi/font/css/materialdesignicons.css'
import './style.css'
import { invoke } from '@tauri-apps/api/tauri'

import App from './App.vue'

// テーマの取得関数
async function getInitialTheme(): Promise<'light' | 'dark' | 'system'> {
    try {
        const theme = await invoke<'light' | 'dark' | 'system'>('get_ui_theme')
        return theme || 'light'
    } catch (error) {
        console.error('テーマ情報の取得に失敗しました:', error)
        return 'light' // デフォルトはlight
    }
}

// Vuetify用カラーパレット
const ballistaTheme = {
    dark: {
        colors: {
            primary: '#DC1E5A',
            'primary-lighten-5': '#FFE5EE',
            'primary-lighten-4': '#FFC4D6',
            'primary-lighten-3': '#FF9EBD',
            'primary-lighten-2': '#FF77A5',
            'primary-lighten-1': '#F9498D',
            'primary-darken-1': '#C21A50',
            'primary-darken-2': '#A81646',
            'primary-darken-3': '#8F123C',
            'primary-darken-4': '#750F32',
            
            secondary: '#2D2D2D',
            'secondary-lighten-5': '#5C5C5C',
            'secondary-lighten-4': '#4D4D4D',
            'secondary-lighten-3': '#3E3E3E',
            'secondary-lighten-2': '#363636',
            'secondary-lighten-1': '#323232',
            'secondary-darken-1': '#252525',
            'secondary-darken-2': '#1E1E1E',
            'secondary-darken-3': '#171717',
            'secondary-darken-4': '#101010',
            
            background: '#1A1A1A',
            surface: '#2D2D2D',
            
            // Material Design準拠の追加カラー
            'surface-variant': '#3E3E3E',     // 読み取り専用入力欄など
            'surface-disabled': '#1E1E1E',    // 無効状態の背景色
            'surface-hover': '#3E3E3E',       // ホバー時の背景色
            'outline': 'rgba(255, 255, 255, 0.12)', // 境界線の色
            
            success: '#4CAF50',
            info: '#2196F3',
            warning: '#FB8C00',
            error: '#FF5252',
            
            'on-background': '#FFFFFF',
            'on-surface': '#FFFFFF',
            'on-primary': '#FFFFFF',
            'on-secondary': '#FFFFFF',
            'on-surface-variant': 'rgba(255, 255, 255, 0.7)', // セカンダリテキスト
            'on-surface-disabled': 'rgba(255, 255, 255, 0.38)', // 無効状態のテキスト
        },
    },
    light: {
        colors: {
            primary: '#DC1E5A',
            'primary-lighten-5': '#FFE5EE',
            'primary-lighten-4': '#FFC4D6',
            'primary-lighten-3': '#FF9EBD',
            'primary-lighten-2': '#FF77A5',
            'primary-lighten-1': '#F9498D',
            'primary-darken-1': '#C21A50',
            'primary-darken-2': '#A81646',
            'primary-darken-3': '#8F123C',
            'primary-darken-4': '#750F32',
            
            secondary: '#424242',
            'secondary-lighten-5': '#F5F5F5',
            'secondary-lighten-4': '#EEEEEE',
            'secondary-lighten-3': '#E0E0E0',
            'secondary-lighten-2': '#BDBDBD',
            'secondary-lighten-1': '#9E9E9E',
            'secondary-darken-1': '#757575',
            'secondary-darken-2': '#616161',
            'secondary-darken-3': '#424242',
            'secondary-darken-4': '#212121',
            
            background: '#F5F5F5',
            surface: '#FFFFFF',
            
            // Material Design準拠の追加カラー
            'surface-variant': 'rgba(0, 0, 0, 0.02)', // 読み取り専用入力欄など
            'surface-disabled': 'rgba(0, 0, 0, 0.04)', // 無効状態の背景色
            'surface-hover': 'rgba(0, 0, 0, 0.04)',    // ホバー時の背景色
            'outline': 'rgba(0, 0, 0, 0.12)',         // 境界線の色
            
            success: '#4CAF50',
            info: '#2196F3',
            warning: '#FB8C00',
            error: '#FF5252',
            
            'on-background': '#212121',
            'on-surface': '#212121',
            'on-primary': '#FFFFFF',
            'on-secondary': '#FFFFFF',
            'on-surface-variant': 'rgba(0, 0, 0, 0.6)', // セカンダリテキスト
            'on-surface-disabled': 'rgba(0, 0, 0, 0.38)', // 無効状態のテキスト
        },
    },
}

// テーマの更新関数をグローバルに公開
export const updateTheme = (theme: 'light' | 'dark' | 'system') => {
    const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    const themeToApply = theme === 'system' ? systemTheme : theme
    
    vuetify.theme.global.name.value = themeToApply
}

// Vuetifyの初期化
const vuetify = createVuetify({
    components,
    directives,
    theme: {
        defaultTheme: 'dark', // デフォルトはdarkテーマ
        themes: ballistaTheme,
    },
})

// アプリケーションの初期化
const app = createApp(App)
app.use(vuetify)
app.mount('#app')

// APIからテーマを取得して適用
getInitialTheme().then(theme => {
    updateTheme(theme)
})

// システムテーマの変更を検知するイベントリスナー（system設定時のみ使用）
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', e => {
    // 現在のテーマがsystemの場合のみ切り替え
    invoke<'light' | 'dark' | 'system'>('get_ui_theme').then(theme => {
        if (theme === 'system') {
            updateTheme('system')
        }
    }).catch(error => {
        console.error('テーマ取得エラー:', error)
    })
})