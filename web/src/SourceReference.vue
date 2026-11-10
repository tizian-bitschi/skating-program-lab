<script setup lang="ts">
import {computed} from 'vue';
import type {Source} from './api';
import type {Locale} from './i18n';
const props=defineProps<{source:Source;locale:Locale}>();
const label=computed(()=>({en:'Open original source',de:'Originalquelle öffnen',ru:'Открыть оригинал источника'})[props.locale]);
</script>
<template><span>{{source.document}} · {{source.version}} · {{source.location}} <a v-if="source.url" :href="source.url" :aria-label="`${label}: ${source.document}, ${source.version}`">{{label}}</a></span></template>
