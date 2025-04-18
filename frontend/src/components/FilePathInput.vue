<template>
    <div class="file-path-container">
        <!-- タイトル部分 -->
        <div class="file-path-title">{{ title }}</div>
        
        <!-- 入力フィールドとボタンのコンテナ -->
        <div class="file-path-input-container">
            <!-- テキスト入力エリア -->
            <input
                type="text"
                v-model="path"
                :disabled="disabled"
                :readonly="readOnly"
                :placeholder="placeholder"
                class="file-path-text-input"
                :class="{ 'read-only': readOnly }"
                @input="handleInputChange"
            />
            
            <!-- ファイル選択ボタン -->
            <button
                @click="handleButtonClick"
                class="file-path-button"
                :disabled="disabled"
            >
                <div class="dots-container">
                    <div class="dot"></div>
                    <div class="dot"></div>
                    <div class="dot"></div>
                </div>
            </button>
        </div>

        <!-- 非表示のファイル入力（カスタム選択関数がない場合に使用） -->
        <input
            v-if="!onButtonClick"
            ref="fileInputRef"
            type="file"
            @change="handleFileChange"
            class="hidden-input"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { useTheme } from 'vuetify';

// テーマの取得
const theme = useTheme();

// テーマカラーの取得関数
const getColor = (colorName: string) => {
    return computed(() => theme.global.current.value.colors[colorName] || '');
};

// テーマから色を取得
const surfaceColor = getColor('surface');
const surfaceVariantColor = getColor('surface-variant');
const surfaceDisabledColor = getColor('surface-disabled');
const surfaceHoverColor = getColor('surface-hover');
const onSurfaceColor = getColor('on-surface');
const onSurfaceVariantColor = getColor('on-surface-variant');
const onSurfaceDisabledColor = getColor('on-surface-disabled');
const outlineColor = getColor('outline');

// プロパティの定義
interface Props {
    title?: string;
    disabled?: boolean;
    readOnly?: boolean;
    modelValue?: string;
    placeholder?: string;
    onButtonClick?: () => void;
}

const props = withDefaults(defineProps<Props>(), {
    title: 'C: Program',
    disabled: false,
    readOnly: false,
    modelValue: '',
    placeholder: 'パスを入力または選択してください',
    onButtonClick: undefined
});

// イベントの定義
const emit = defineEmits(['update:modelValue']);

// 内部状態
const path = ref(props.modelValue);
const fileInputRef = ref<HTMLInputElement | null>(null);

// プロパティの変更を監視して内部状態を更新
watch(() => props.modelValue, (newValue) => {
    path.value = newValue;
});

// 入力変更のハンドラ
const handleInputChange = (e: Event) => {
    if (props.readOnly) return;
    
    const target = e.target as HTMLInputElement;
    path.value = target.value;
    emit('update:modelValue', target.value);
};

// ボタンクリックのハンドラ
const handleButtonClick = () => {
    if (props.onButtonClick) {
        // カスタム選択関数があれば使用
        props.onButtonClick();
    } else {
        // なければデフォルトのファイル選択を使用
        fileInputRef.value?.click();
    }
};

// ファイル選択のハンドラ
const handleFileChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
        const filePath = target.files[0].name; // 実際のブラウザでは完全なパスは取得できませんが、デモ用
        path.value = filePath;
        emit('update:modelValue', filePath);
    }
};
</script>

<style scoped>
.file-path-container {
    position: relative;
    width: 100%;
    margin-top: 10px;
}

.file-path-title {
    position: absolute;
    top: -10px;
    left: 10px;
    padding: 0 6px;
    font-size: 12px;
    z-index: 1;
    background-color: v-bind('surfaceColor');
    color: v-bind('onSurfaceVariantColor');
}

.file-path-input-container {
    display: flex;
    border-radius: 8px;
    overflow: hidden;
    height: 40px;
    border: 1px solid v-bind('outlineColor');
}

.file-path-text-input {
    flex: 1;
    border: none;
    padding: 0 12px;
    outline: none;
    font-size: 14px;
    background-color: v-bind('surfaceColor');
    color: v-bind('onSurfaceColor');
}

.file-path-text-input.read-only {
    background-color: v-bind('surfaceVariantColor');
    cursor: default;
}

.file-path-text-input:disabled {
    background-color: v-bind('surfaceDisabledColor');
    color: v-bind('onSurfaceDisabledColor');
}

.file-path-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    border: none;
    cursor: pointer;
    background-color: v-bind('surfaceColor');
    border-left: 1px solid v-bind('outlineColor');
}

.file-path-button:hover {
    background-color: v-bind('surfaceHoverColor');
}

.file-path-button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
}

.dots-container {
    display: flex;
    gap: 3px;
}

.dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background-color: v-bind('onSurfaceVariantColor');
}

.hidden-input {
    display: none;
}
</style>