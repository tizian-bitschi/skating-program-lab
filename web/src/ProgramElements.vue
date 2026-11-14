<script setup lang="ts">
import {computed} from 'vue';
import {type Locale,formName} from './i18n';
import {connectionIdentity,connectionText} from './connection-text';
export interface ResultEntry {elementKey:string;code:string;requiredId:string|null}
export interface ResultAssessment {totalHundredths:number;elements:{creditedHundredths:number|null;parts:{form:string;level:number|null}[]}[]}
const props=defineProps<{indices?:number[];entries:ResultEntry[];assessment:ResultAssessment;locale:Locale;excludedConnections:string[];stale:boolean;retained:string;added:string}>();
const emit=defineEmits<{exclude:[key:string]}>();
const labels=computed(()=>connectionText[props.locale]);
const excluded=computed(()=>new Set(props.excludedConnections.map(connectionIdentity)));
const number=(value:number|null|undefined)=>value==null?'—':new Intl.NumberFormat(props.locale,{minimumFractionDigits:2,maximumFractionDigits:2}).format(value/100);
</script>
<template>
<ol class="suggestion-elements">
 <li v-for="index in (indices??entries.map((_,index)=>index))" :key="index">
  <div class="suggestion-element-heading"><span>{{assessment.elements[index]?.parts.map(part=>`${formName(locale,part.form)}${part.level==null?'':` ${part.level===0?'B':part.level}`}`).join(' + ')}}</span><strong>{{number(assessment.elements[index]?.creditedHundredths)}}<small v-if="stale">{{labels.outdated}}</small></strong></div>
  <code>{{entries[index]!.code}}</code><small class="provenance">{{entries[index]!.requiredId?retained:added}}</small>
  <template v-if="(assessment.elements[index]?.parts.length??0)>1">
   <small v-if="entries[index]!.elementKey.endsWith(':true')">{{labels.sequence}}</small>
   <button data-testid="exclude-result" :aria-disabled="excluded.has(connectionIdentity(entries[index]!.elementKey))" @click="!excluded.has(connectionIdentity(entries[index]!.elementKey))&&emit('exclude',entries[index]!.elementKey)">{{excluded.has(connectionIdentity(entries[index]!.elementKey))?labels.excluded:labels.exclude}} {{entries[index]!.code}}</button>
  </template>
 </li>
</ol>
</template>
