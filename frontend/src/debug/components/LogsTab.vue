/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<template>
    <v-card-text>
        <v-form>
            <v-textarea :model-value="logs" label="実行ログ" readonly variant="outlined" rows="20"
                class="log-area" style="white-space: pre-wrap;" persistent-placeholder hide-details="auto"></v-textarea>
        </v-form>
    </v-card-text>
</template>

<script setup lang="ts">
import { defineProps } from 'vue';

defineProps<{
    logs: string;
}>();
</script>

<style scoped>
.log-area {
    font-family: monospace;
    white-space: pre;
    font-size: 0.8rem;
}
</style>
