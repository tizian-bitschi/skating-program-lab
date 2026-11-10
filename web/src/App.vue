<script setup lang="ts">
import SourceReference from './SourceReference.vue';
import { computed,nextTick,onMounted,ref,watch } from 'vue';
import ManualEditor from './ManualEditor.vue';
import ConnectionBuilder from './ConnectionBuilder.vue';
import PlanningSearch from './PlanningSearch.vue';
import { type ProgramEntry,type Form,type Profile,type Repertoire,type Limit,emptyLimits,fetchJson } from './api';
import { type Locale,type TextKey,browserLocale,formName,text } from './i18n';
import {parseWorkspace,serializeWorkspace,workspaceKey,workspaceText,type Workspace} from './workspace';
const editor=ref<InstanceType<typeof ManualEditor>>(),entries=ref<ProgramEntry[]>([]),complete=ref(false),excludedConnections=ref<string[]>([]),allowMixedDirections=ref(false);
const loadingWorkspace=ref(true),ready=ref(false),storageState=ref<'saved'|'access'|'save'|'invalid'|'context'>();
function onProgramChange(next:ProgramEntry[],fullyDescribed:boolean){entries.value=next;complete.value=fullyDescribed;}
const locale=ref<Locale>(browserLocale());
const t=(key:TextKey)=>text(locale.value,key);
const profiles=ref<Profile[]>([]),selected=ref(''),repertoire=ref<Repertoire>(),limits=ref(emptyLimits());
const pending=ref(false),failure=ref(false);let revision=0;
const mixerOpen=ref(!matchMedia('(max-width:1099px)').matches);
const mixerToggle=computed(()=>({en:mixerOpen.value?'Hide controls':'Show all controls',de:mixerOpen.value?'Regler ausblenden':'Alle Regler anzeigen',ru:mixerOpen.value?'Скрыть регуляторы':'Показать все регуляторы'}[locale.value]));
const profile=computed(()=>profiles.value.find(p=>p.context.profile===selected.value));
const groups=[['jump','jumps'],['spin','spins'],['other','other']] as const;
function limit(form:Form):Limit|undefined {if(form.family==='jump')return limits.value.jumps[form.id];if(form.family==='spin')return limits.value.spins[form.id];if(form.family==='step')return limits.value.steps;return {enabled:limits.value.unlevelledElements[form.id]??true};}
function setLimit(form:Form,value:Limit){if(form.family==='jump')limits.value.jumps[form.id]=value;else if(form.family==='spin')limits.value.spins[form.id]=value;else if(form.family==='step')limits.value.steps=value;else limits.value.unlevelledElements[form.id]=value.enabled;}
const faderPreview=ref<Record<string,number>>({});
function faderStops(form:Form){return form.family==='unlevelled'?[0]:form.eligibleLevels;}
function faderValue(form:Form){
 if(!form.available)return 0;
 if(faderPreview.value[form.id]!==undefined)return faderPreview.value[form.id]!;
 const setting=limit(form);if(setting?.enabled===false)return 0;
 const stops=faderStops(form);
 const eligible=setting?.maximum===undefined?stops:stops.filter(n=>n<=setting.maximum!);
 return eligible.length?stops.indexOf(eligible[eligible.length-1]!)+1:0;
}
function previewFader(form:Form,event:Event){faderPreview.value[form.id]=Number((event.target as HTMLInputElement).value);}
function fader(form:Form,event:Event){
 const value=Number((event.target as HTMLInputElement).value);delete faderPreview.value[form.id];
 if(value===0){setLimit(form,{...limit(form),enabled:false});return;}
 const maximum=faderStops(form)[value-1];if(maximum===undefined)return;
 setLimit(form,{enabled:true,...(form.family==='unlevelled'?{}:{maximum})});
}
function faderKey(form:Form,event:KeyboardEvent){
 if(event.key!=='Home'&&event.key!=='End')return;
 event.preventDefault();
 (event.target as HTMLInputElement).value=String(event.key==='Home'?0:faderStops(form).length);
 fader(form,event);
}
function faderText(form:Form){
 if(!form.available)return t('unavailable');
 const value=faderValue(form);if(value===0)return t('off');
 if(form.family==='unlevelled')return t('on');
 const maximum=faderStops(form)[value-1];return `${t('maximum')}: ${maximum===0?t('basic'):maximum}`;
}
function faderReadout(form:Form){
 if(!form.available)return '—';
 const value=faderValue(form);if(value===0)return t('off');
 if(form.family==='unlevelled')return t('on');
 const maximum=faderStops(form)[value-1];return maximum===0?'B':maximum;
}
function forms(group:string){return repertoire.value?.forms.filter(f=>group==='other'?['step','unlevelled'].includes(f.family):f.family===group)??[];}
function profileName(p:Profile){ const known:Record<string,TextKey>={'bronze':'bronze','silver':'silver','gold':'gold','masters':'masters','women':'women','men':'men'};const key=known[p.label];return key?(['bronze','silver','gold','masters'].includes(p.label)?`${t('adult')} ${t(key)}`:t(key)):p.label; }
async function refresh(){if(!profile.value)return;const ownRevision=++revision;pending.value=true;failure.value=false;try{const next=await fetchJson<Repertoire>('/api/v1/repertoire',{requestId:crypto.randomUUID(),revision:ownRevision,context:profile.value.context,personalLimits:limits.value});if(ownRevision===revision)repertoire.value=next;}catch{if(ownRevision===revision)failure.value=true;}finally{if(ownRevision===revision)pending.value=false;}}
function saveWorkspace(){
 if(!ready.value||!profile.value)return;
 try{localStorage.setItem(workspaceKey,serializeWorkspace({version:1,context:profile.value.context,entries:entries.value,complete:complete.value,personalLimits:limits.value,excludedConnections:excludedConnections.value,allowMixedDirections:allowMixedDirections.value,locale:locale.value}));storageState.value='saved';}catch{storageState.value='save';ready.value=false;}
}
async function restoreWorkspace(){
 ready.value=false;storageState.value=undefined;
 let raw:string|null;
 try{raw=localStorage.getItem(workspaceKey);}catch{storageState.value='access';return;}
 let saved:Workspace|undefined;
 if(raw!==null){try{saved=parseWorkspace(raw);locale.value=saved.locale;}catch{storageState.value='invalid';return;}}
 if(saved){
 const exact=profiles.value.find(p=>p.context.profile===saved.context.profile&&p.context.ruleEdition===saved.context.ruleEdition&&p.context.scoringTable===saved.context.scoringTable);
 if(!exact){storageState.value='context';return;}
 try{
 const restored=await fetchJson<Repertoire>('/api/v1/repertoire',{requestId:crypto.randomUUID(),revision:0,context:saved.context,personalLimits:saved.personalLimits});
 const known=(family:string)=>restored.forms.filter(f=>f.family===family).map(f=>f.id);
 if(Object.keys(saved.personalLimits.jumps).some(k=>!known('jump').includes(k))||Object.keys(saved.personalLimits.spins).some(k=>!known('spin').includes(k))||Object.keys(saved.personalLimits.unlevelledElements).some(k=>!known('unlevelled').includes(k)))throw new Error('invalid_workspace');
 if(saved.entries.some(e=>!e.elementKey.endsWith(`:${e.code}:true`)&&!e.elementKey.endsWith(`:${e.code}:false`)))throw new Error('invalid_workspace');
 await fetchJson('/api/v1/program-assessments',{requestId:crypto.randomUUID(),revision:0,context:saved.context,personalLimits:saved.personalLimits,entries:saved.entries.map(({id,elementKey})=>({id,elementKey})),submission:saved.complete?'complete':'draft',excludedConnections:saved.excludedConnections,allowMixedDirections:saved.allowMixedDirections});
 }catch{storageState.value='context';return;}
 selected.value=exact.context.profile;limits.value=saved.personalLimits;excludedConnections.value=saved.excludedConnections;allowMixedDirections.value=saved.allowMixedDirections;
 editor.value?.replaceEntries(saved.entries,saved.complete);
 }else selected.value=profiles.value[0]?.context.profile??'';
 await nextTick();ready.value=true;saveWorkspace();
}
async function replaceWorkspace(){
 // Only this explicit action replaces a rejected snapshot. setItem stays atomic on quota failure.
 selected.value=profiles.value[0]?.context.profile??'';limits.value=emptyLimits();excludedConnections.value=[];allowMixedDirections.value=false;editor.value?.replaceEntries([],false);
 await nextTick();ready.value=true;saveWorkspace();await focusAfterRecovery();
}
async function focusAfterRecovery(){await nextTick();if(storageState.value==='saved')document.querySelector<HTMLSelectElement>('.navbar select')?.focus();}
async function retryWorkspace(){if(storageState.value==='save'){ready.value=true;saveWorkspace();}else await restoreWorkspace();await focusAfterRecovery();}
async function initialize(){loadingWorkspace.value=true;failure.value=false;try{profiles.value=await fetchJson<Profile[]>('/api/v1/profiles');await restoreWorkspace();}catch{failure.value=true;}finally{loadingWorkspace.value=false;}}
watch([selected,limits,entries,complete,excludedConnections,allowMixedDirections,locale],saveWorkspace,{deep:true});
watch(selected,()=>{faderPreview.value={};});
watch([selected,limits],refresh,{deep:true});watch(locale,value=>{document.documentElement.lang=value;},{immediate:true});onMounted(initialize);
</script>
<template>
 <header class="navbar"><strong>Skating Program Lab</strong><label>{{t('category')}} · 2026/27 <select v-model="selected" :disabled="loadingWorkspace" :aria-label="t('category')"><option v-for="p in profiles" :key="p.context.profile" :value="p.context.profile">{{profileName(p)}}</option></select></label><label>{{t('language')}} <select v-model="locale" aria-label="Language"><option value="de">Deutsch</option><option value="en">English</option><option value="ru">Русский</option></select></label></header>
 <main>
 <p v-if="storageState" data-testid="workspace-storage" :class="{'storage-saved':storageState==='saved'}" :role="storageState==='saved'?'status':'alert'">{{workspaceText[locale][storageState]}} <template v-if="storageState!=='saved'"><button @click="retryWorkspace">{{workspaceText[locale].retry}}</button> <button @click="replaceWorkspace">{{workspaceText[locale].replace}}</button></template></p>
 <nav class="workspace-links"><a href="#program">{{t('program')}}</a><a href="#suggestions">{{t('results')}}</a></nav>
 <section class="mixer" :aria-label="t('limits')" :aria-busy="pending"><div class="mixer-heading"><h1>{{t('limits')}}</h1><button class="mixer-toggle" :aria-expanded="mixerOpen" aria-controls="mixer-controls" @click="mixerOpen=!mixerOpen">{{mixerToggle}}</button></div>
 <div v-if="failure" role="alert">{{t('failure')}} <button @click="profiles.length?refresh():initialize()">{{t('retry')}}</button></div>
 <fieldset v-show="mixerOpen" id="mixer-controls" :disabled="loadingWorkspace" class="workspace-guard"><div class="mixer-groups"><section v-for="[group,title] in groups" :key="group" :class="['group',group]"><h2>{{t(title)}}</h2><div class="channels">
 <div v-for="form in forms(group)" :key="form.id" :data-testid="`form-${form.id}`" :data-family="form.family" :class="['channel',{'unavailable':!form.available}]">
 <label class="form-name" :for="`fader-${form.id}`" :title="formName(locale,form.id)">{{form.id}}<span>{{formName(locale,form.id)}}</span></label>
 <input :id="`fader-${form.id}`" class="fader" type="range" min="0" :max="Math.max(1,faderStops(form).length)" step="1" :value="faderValue(form)" :disabled="!form.available||repertoire?.context.profile!==profile?.context.profile" :aria-label="formName(locale,form.id)" :aria-valuetext="faderText(form)" :data-testid="`fader-${form.id}`" @input="previewFader(form,$event)" @change="fader(form,$event)" @keydown="faderKey(form,$event)"><span class="fader-readout" :title="faderText(form)">{{faderReadout(form)}}</span>
 <small v-if="!form.available">{{t('unavailable')}}</small>
 <small v-else-if="form.family!=='unlevelled'">{{form.permittedLevels.map(n=>n===0?'B':n).join(' · ')||'—'}}</small>
 </div></div></section></div>
 </fieldset><p class="status" role="status">{{pending?t('loading'):`${repertoire?.elements.length??0} ${t('permitted')}`}}</p></section>
 <div class="workspace"><section><h2 id="program" tabindex="-1">{{t('program')}}</h2><fieldset :disabled="loadingWorkspace" class="editor-guard"><ManualEditor :excluded-connections="excludedConnections" :allow-mixed-directions="allowMixedDirections" ref="editor" @change="onProgramChange" :context="profile?.context" :limits="limits" :repertoire="repertoire" :locale="locale" /><ConnectionBuilder purpose="exclude" :context="profile?.context" :limits="limits" :locale="locale" :excluded-connections="excludedConnections" :allow-mixed-directions="allowMixedDirections" @change="excludedConnections=$event" /></fieldset></section><section><h2 id="suggestions" tabindex="-1">{{t('results')}}</h2><PlanningSearch :context="loadingWorkspace?undefined:profile?.context" :limits="limits" :entries="entries" :locale="locale" :excluded-connections="excludedConnections" :allow-mixed-directions="allowMixedDirections" @adopt="editor?.replaceEntries($event,true)" /></section></div>
 <details class="sources"><summary>{{t('source')}}</summary><p v-if="profile">{{profile.context.profile}} · {{profile.context.ruleEdition}} · {{profile.context.scoringTable}}</p><ul><li v-for="source in profile?.sources" :key="source.document+source.location"><SourceReference :source="source" :locale="locale" /></li></ul></details>

 </main>
</template>
