<script setup lang="ts">
import {restoreOccurrences} from './program-entries';
import {abilityText} from './ability-text';
import SourceReference from './SourceReference.vue';
import {connectionIdentity,connectionCode,connectionText} from './connection-text';
import { computed,nextTick,onMounted,ref,watch } from 'vue';
import ManualEditor from './ManualEditor.vue';
import ConnectionBuilder from './ConnectionBuilder.vue';
import PlanningSearch from './PlanningSearch.vue';
import { type ProgramEntry,type Form,type Profile,type Repertoire,type Limit,emptyLimits,fetchJson } from './api';
import { type Locale,type TextKey,browserLocale,formName,text } from './i18n';
import {parseWorkspace,serializeWorkspace,workspaceKey,workspaceText,type Workspace} from './workspace';
const editor=ref<InstanceType<typeof ManualEditor>>(),entries=ref<ProgramEntry[]>([]),complete=ref(false),excludedConnections=ref<string[]>([]),allowMixedDirections=ref(false);
const exclusionLabels=computed(()=>connectionText[locale.value]);
const exclusionNotice=ref<{key:string;removed:number}| 'undone'>();
const exclusionFeedback=computed(()=>{
 const notice=exclusionNotice.value;if(!notice)return '';if(notice==='undone')return exclusionLabels.value.undone;
 return `${exclusionLabels.value.excluded}: ${connectionCode(notice.key)}. ${exclusionLabels.value.removed}: ${notice.removed}.`;
});
const exclusionUndo=ref<{key:string;before:ProgramEntry[];removed:ProgramEntry[]}>();
function replaceProgram(next:ProgramEntry[]){entries.value=next;editor.value?.editEntries(next);}
function changeExclusions(keys:string[]){
 abilityUndo.value=undefined;abilityNotice.value=undefined;
 const added=keys.filter(key=>!excludedConnections.value.includes(key));
 exclusionUndo.value=undefined;
 if(!added.length)exclusionNotice.value=undefined;
 excludedConnections.value=keys;
 for(const key of added){
  const before=entries.value.map(entry=>({...entry}));
  const removed=before.filter(entry=>connectionIdentity(entry.elementKey)===connectionIdentity(key));
  replaceProgram(before.filter(entry=>!removed.some(item=>item.id===entry.id)));
  exclusionUndo.value={key,before,removed};
  exclusionNotice.value={key,removed:removed.length};
 }
}
function excludeConnection(key:string){
 if(!excludedConnections.value.some(saved=>connectionIdentity(saved)===connectionIdentity(key)))changeExclusions([...excludedConnections.value,key]);
}
async function undoExclusion(){
 const undo=exclusionUndo.value;if(!undo)return;
 const restored=restoreOccurrences(entries.value,undo.before,undo.removed);
 excludedConnections.value=excludedConnections.value.filter(key=>key!==undo.key);
 replaceProgram(restored);exclusionUndo.value=undefined;exclusionNotice.value='undone';
 await nextTick();document.getElementById('exclusion-feedback')?.focus({preventScroll:true});
}
const loadingWorkspace=ref(true),ready=ref(false),storageState=ref<'saved'|'access'|'save'|'invalid'|'context'>();
function onProgramChange(next:ProgramEntry[],fullyDescribed:boolean){entries.value=next;complete.value=fullyDescribed;}
const locale=ref<Locale>(browserLocale());
const t=(key:TextKey)=>text(locale.value,key);
const profiles=ref<Profile[]>([]),selected=ref(''),repertoire=ref<Repertoire>(),limits=ref(emptyLimits());
const pending=ref(false),failure=ref(false);let revision=0;
type Panel='ability'|'program'|'suggestions'|'exclusions';
const panels:Panel[]=['ability','program','suggestions','exclusions'];
const activePanel=ref<Panel>('ability'),search=ref<InstanceType<typeof PlanningSearch>>();
const panelCopy={
 en:{more:'More options',mixed:'Allow mixed rotation directions within a connection',navigation:'Planning areas',ability:'Ability',program:'My program',suggestions:'Suggestions',exclusions:'Exclusions',discover:'Calculate suggestions',specify:'Specify elements first',complete:'Calculate around required elements',intro:'Set your ability, then choose how to start.',required:'Added elements are required by default. Release an occurrence to make it interchangeable.'},
 de:{more:'Weitere Optionen',mixed:'Verschiedene Drehrichtungen innerhalb einer Verbindung zulassen',navigation:'Planungsbereiche',ability:'Können',program:'Mein Programm',suggestions:'Vorschläge',exclusions:'Ausschlüsse',discover:'Vorschläge berechnen',specify:'Zuerst Elemente festlegen',complete:'Um erforderliche Elemente ergänzen',intro:'Stelle dein Können ein und wähle deinen Einstieg.',required:'Hinzugefügte Elemente sind zunächst erforderlich. Gib ein Vorkommen frei, um es austauschbar zu machen.'},
 ru:{more:'Дополнительные настройки',mixed:'Разрешить разные направления вращения в соединении',navigation:'Разделы планирования',ability:'Навыки',program:'Моя программа',suggestions:'Варианты',exclusions:'Исключения',discover:'Рассчитать варианты',specify:'Сначала задать элементы',complete:'Дополнить обязательные элементы',intro:'Настройте навыки и выберите, с чего начать.',required:'Добавленные элементы обязательны. Снимите отметку с отдельного элемента, чтобы разрешить его замену.'}
};
const panelLabels=computed(()=>panelCopy[locale.value]);
const readingPositions:Partial<Record<Panel,number>>={};
async function showPanel(panel:Panel){
 const phone=matchMedia('(max-width:1099px)').matches;
 if(phone)readingPositions[activePanel.value]=window.scrollY;
 activePanel.value=panel;await nextTick();
 if(phone)window.scrollTo(0,readingPositions[panel]??0);
 else document.getElementById(`area-${panel}`)?.scrollIntoView({block:'start'});
 document.getElementById(`${phone?'panel':'area'}-${panel}`)?.focus({preventScroll:true});
}
async function calculate(){await showPanel('suggestions');await search.value?.calculate();}
async function adoptProgram(next:ProgramEntry[]){
 abilityUndo.value=undefined;changedAbilityForms.clear();exclusionUndo.value=undefined;editor.value?.adoptEntries(next);await showPanel('program');
 const panel=document.getElementById('area-program');panel?.scrollIntoView({block:'start'});panel?.focus({preventScroll:true});
}
const profile=computed(()=>profiles.value.find(p=>p.context.profile===selected.value));
const groups=[['jump','jumps'],['spin','spins'],['other','other']] as const;
function limit(form:Form):Limit|undefined {if(form.family==='jump')return limits.value.jumps[form.id];if(form.family==='spin')return limits.value.spins[form.id];if(form.family==='step')return limits.value.steps;const enabled=limits.value.unlevelledElements[form.id];return enabled===undefined?undefined:{enabled};}
function setLimit(form:Form,value?:Limit){
 if(form.family==='step'){limits.value.steps=value;return;}
 if(form.family==='unlevelled'){if(value)limits.value.unlevelledElements[form.id]=value.enabled;else delete limits.value.unlevelledElements[form.id];return;}
 const settings=form.family==='jump'?limits.value.jumps:limits.value.spins;
 if(value)settings[form.id]=value;else delete settings[form.id];
}
const changedAbilityForms=new Set<string>();
const abilityUndo=ref<{form:Form;previous?:Limit;before:ProgramEntry[];removed:ProgramEntry[]}>();
const abilityNotice=ref<{formId:string;removed:ProgramEntry[]}| 'undone'>();
const abilityLabels=computed(()=>abilityText[locale.value]);
const abilityFeedback=computed(()=>{
 const notice=abilityNotice.value;if(!notice)return '';if(notice==='undone')return abilityLabels.value.undone;
 return `${abilityLabels.value.changed}: ${formName(locale.value,notice.formId)}. ${abilityLabels.value.removed}: ${notice.removed.length}${notice.removed.length?` (${notice.removed.map(entry=>entry.code).join(', ')})`:''}.`;
});
function commitAbility(form:Form,value:Limit){
 exclusionNotice.value=undefined;
 const previous=limit(form);if(JSON.stringify(previous)===JSON.stringify(value))return;
 abilityUndo.value={form,previous:previous?{...previous}:undefined,before:entries.value.map(entry=>({...entry})),removed:[]};
 abilityNotice.value={formId:form.id,removed:[]};
 if(!value.enabled||previous?.enabled!==false&&value.maximum!==undefined&&(previous?.maximum===undefined||value.maximum<previous.maximum))changedAbilityForms.add(form.id);
 setLimit(form,value);
}
function applyAbilityAssessment(assessment:{personalConflicts:{entryId?:string;translationKey?:string;setting?:string|string[]}[]}){
 if(!changedAbilityForms.size)return;
 const conflicts=assessment.personalConflicts.filter(f=>['disabled','maximum'].includes(f.translationKey??''));
 const settings=(f:typeof conflicts[number])=>Array.isArray(f.setting)?f.setting:[f.setting];
 const removed=entries.value.filter(entry=>conflicts.some(f=>f.entryId===entry.id&&settings(f).some(id=>id&&changedAbilityForms.has(id))));
 const undo=abilityUndo.value;
 if(undo){
  undo.removed=removed.filter(entry=>undo.before.some(original=>original.id===entry.id)&&conflicts.some(f=>f.entryId===entry.id&&settings(f).includes(undo.form.id))&&!conflicts.some(f=>f.entryId===entry.id&&settings(f).some(id=>id!==undo.form.id)));
  abilityNotice.value={formId:undo.form.id,removed};
 }
 changedAbilityForms.clear();
 if(removed.length)replaceProgram(entries.value.filter(entry=>!removed.some(item=>item.id===entry.id)));
}
async function undoAbility(){
 const undo=abilityUndo.value;if(!undo)return;
 abilityUndo.value=undefined;setLimit(undo.form,undo.previous);changedAbilityForms.add(undo.form.id);
 replaceProgram(restoreOccurrences(entries.value,undo.before,undo.removed));abilityNotice.value='undone';
 await nextTick();document.getElementById('ability-feedback')?.focus({preventScroll:true});
}
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
 if(value===0){commitAbility(form,{...limit(form),enabled:false});return;}
 const maximum=faderStops(form)[value-1];if(maximum===undefined)return;
 commitAbility(form,{enabled:true,...(form.family==='unlevelled'?{}:{maximum})});
}
function faderKey(form:Form,event:KeyboardEvent){
 if(event.key!=='Home'&&event.key!=='End')return;
 event.preventDefault();
 (event.target as HTMLInputElement).value=String(event.key==='Home'?0:faderStops(form).length);
 fader(form,event);
}
function stopText(form:Form,maximum:number){
 if(form.family==='jump')return [abilityLabels.value.single,abilityLabels.value.double,abilityLabels.value.triple,abilityLabels.value.quadruple,abilityLabels.value.quintuple][maximum-1]??String(maximum);
 return maximum===0?t('basic'):String(maximum);
}
function faderText(form:Form){
 if(!form.available)return t('unavailable');
 const value=faderValue(form);if(value===0)return t('off');
 if(form.family==='unlevelled')return t('on');
 const maximum=faderStops(form)[value-1]!;return `${t('maximum')}: ${stopText(form,maximum)}`;
}
function faderReadout(form:Form){
 if(!form.available)return '—';
 const value=faderValue(form);if(value===0)return t('off');
 if(form.family==='unlevelled')return t('on');
 return stopText(form,faderStops(form)[value-1]!);
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
watch(selected,()=>{faderPreview.value={};changedAbilityForms.clear();abilityUndo.value=undefined;abilityNotice.value=undefined;});
watch([selected,limits,allowMixedDirections],()=>{exclusionUndo.value=undefined;},{deep:true});
watch([selected,limits],refresh,{deep:true});watch(locale,value=>{document.documentElement.lang=value;},{immediate:true});onMounted(initialize);
</script>
<template>
 <header class="navbar"><strong>Skating Program Lab</strong><label>{{t('category')}} · 2026/27 <select v-model="selected" :disabled="loadingWorkspace" :aria-label="t('category')"><option v-for="p in profiles" :key="p.context.profile" :value="p.context.profile">{{profileName(p)}}</option></select></label><label>{{t('language')}} <select v-model="locale" aria-label="Language"><option value="de">Deutsch</option><option value="en">English</option><option value="ru">Русский</option></select></label></header>
 <main>
 <p v-if="storageState" data-testid="workspace-storage" :class="{'storage-saved':storageState==='saved'}" :role="storageState==='saved'?'status':'alert'">{{workspaceText[locale][storageState]}} <template v-if="storageState!=='saved'"><button @click="retryWorkspace">{{workspaceText[locale].retry}}</button> <button @click="replaceWorkspace">{{workspaceText[locale].replace}}</button></template></p>
 <div v-if="exclusionFeedback" class="exclusion-feedback"><p id="exclusion-feedback" tabindex="-1" data-testid="exclusion-feedback" role="status">{{exclusionFeedback}}</p><button v-if="exclusionUndo" data-testid="undo-exclusion" @click="undoExclusion">{{exclusionLabels.undo}}</button></div>
 <div v-if="abilityFeedback" class="ability-feedback"><p id="ability-feedback" tabindex="-1" data-testid="ability-feedback" role="status">{{abilityFeedback}}</p><button v-if="abilityUndo" data-testid="undo-ability" @click="undoAbility">{{abilityLabels.undo}}</button></div>
 <nav class="panel-navigation" :aria-label="panelLabels.navigation"><button v-for="panel in panels" :key="panel" :id="`panel-${panel}`" :data-testid="`panel-${panel}`" :aria-current="activePanel===panel?'page':undefined" :aria-controls="`area-${panel}`" @click="showPanel(panel)">{{panelLabels[panel]}}</button></nav>
 <section id="area-ability" tabindex="-1" :class="['mixer','planning-panel',{'active-panel':activePanel==='ability'}]" :aria-label="panelLabels.ability" :aria-busy="pending"><div class="mixer-heading"><h1>{{panelLabels.ability}}</h1></div><p>{{panelLabels.intro}}</p><div class="entry-paths"><button data-testid="discover-programs" :disabled="loadingWorkspace||pending||!profile||search?.busy" @click="calculate">{{panelLabels.discover}}</button><button data-testid="specify-elements" :disabled="loadingWorkspace" @click="showPanel('program')">{{panelLabels.specify}}</button></div>
 <div v-if="failure" role="alert">{{t('failure')}} <button @click="profiles.length?refresh():initialize()">{{t('retry')}}</button></div>
 <details class="rotation-options"><summary>{{panelLabels.more}}</summary><label class="rotation-preference"><input type="checkbox" role="switch" v-model="allowMixedDirections" :disabled="loadingWorkspace">{{panelLabels.mixed}}</label></details>
 <fieldset id="mixer-controls" :disabled="loadingWorkspace" class="workspace-guard"><div class="mixer-groups"><section v-for="[group,title] in groups" :key="group" :class="['group',group]"><h2>{{t(title)}}</h2><div class="channels">
 <div v-for="form in forms(group)" :key="form.id" :data-testid="`form-${form.id}`" :data-family="form.family" :class="['channel',{'unavailable':!form.available}]">
 <label class="form-name" :for="`fader-${form.id}`" :title="formName(locale,form.id)">{{formName(locale,form.id)}}<span>{{form.id}}</span></label>
 <input :id="`fader-${form.id}`" class="fader" type="range" min="0" :max="Math.max(1,faderStops(form).length)" step="1" :value="faderValue(form)" :disabled="!form.available||repertoire?.context.profile!==profile?.context.profile" :aria-label="formName(locale,form.id)" :aria-valuetext="faderText(form)" :data-testid="`fader-${form.id}`" @input="previewFader(form,$event)" @change="fader(form,$event)" @keydown="faderKey(form,$event)"><span class="fader-readout" :title="faderText(form)">{{faderReadout(form)}}</span>
 <small v-if="!form.available">{{t('unavailable')}}</small>
 <small v-else>{{(form.family==='unlevelled'?[t('off'),t('on')]:[t('off'),...form.eligibleLevels.map(n=>stopText(form,n))]).join(' · ')}}</small>
 </div></div></section></div>
 </fieldset><p class="status" role="status">{{pending?t('loading'):`${repertoire?.elements.length??0} ${t('permitted')}`}}</p></section>
 <div class="workspace">
 <section id="area-program" tabindex="-1" :class="['planning-panel',{'active-panel':activePanel==='program'}]" :aria-label="panelLabels.program"><h2 id="program">{{panelLabels.program}}</h2><p>{{panelLabels.required}}</p><fieldset :disabled="loadingWorkspace" class="editor-guard"><ManualEditor :excluded-connections="excludedConnections" :allow-mixed-directions="allowMixedDirections" ref="editor" @change="onProgramChange" @assessed="applyAbilityAssessment" :context="profile?.context" :limits="limits" :repertoire="repertoire" :locale="locale" /></fieldset><button data-testid="complete-program" :disabled="loadingWorkspace||!profile||search?.busy" @click="calculate">{{panelLabels.complete}}</button></section>
 <section id="area-suggestions" tabindex="-1" :class="['planning-panel',{'active-panel':activePanel==='suggestions'}]" :aria-label="panelLabels.suggestions"><h2 id="suggestions">{{panelLabels.suggestions}}</h2><PlanningSearch ref="search" :context="loadingWorkspace?undefined:profile?.context" :limits="limits" :entries="entries" :locale="locale" :excluded-connections="excludedConnections" :allow-mixed-directions="allowMixedDirections" @adopt="adoptProgram" @exclude="excludeConnection" /></section>
 <section id="area-exclusions" tabindex="-1" :class="['planning-panel',{'active-panel':activePanel==='exclusions'}]" :aria-label="panelLabels.exclusions"><h2>{{panelLabels.exclusions}}</h2><fieldset :disabled="loadingWorkspace" class="editor-guard"><ConnectionBuilder purpose="exclude" :context="profile?.context" :limits="limits" :locale="locale" :excluded-connections="excludedConnections" :allow-mixed-directions="allowMixedDirections" @change="changeExclusions" /></fieldset></section>
 </div>
 <details class="sources"><summary>{{t('source')}}</summary><p v-if="profile">{{profile.context.profile}} · {{profile.context.ruleEdition}} · {{profile.context.scoringTable}}</p><ul><li v-for="source in profile?.sources" :key="source.document+source.location"><SourceReference :source="source" :locale="locale" /></li></ul></details>

 </main>
</template>
