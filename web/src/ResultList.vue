<script setup lang="ts" generic="T extends {id:string}">
import {computed,ref,watch} from 'vue';
import type {Locale} from './i18n';
const props=defineProps<{results:T[];locale:Locale}>();
const page=ref(0),pageSize=20;
const labels=computed(()=>({en:{previous:'Previous programs',next:'Next programs',page:'Programs',number:'Page',confirmed:'confirmed'},de:{previous:'Vorherige Programme',next:'Weitere Programme',page:'Programme',number:'Seite',confirmed:'bestätigt'},ru:{previous:'Предыдущие программы',next:'Следующие программы',page:'Программы',number:'Страница',confirmed:'подтверждено'}}[props.locale]));
const visible=computed(()=>props.results.slice(page.value*pageSize,(page.value+1)*pageSize));
watch(()=>props.results[0]?.id,()=>{page.value=0;});
function move(delta:number){page.value+=delta;}
function jump(event:Event){const input=event.target as HTMLInputElement;if(input.validity.valid)page.value=Math.max(0,Math.min(Math.ceil(props.results.length/pageSize)-1,Number(input.value)-1));}
</script>
<template>
<div class="result-browser">
 <nav v-if="results.length>pageSize" class="result-pages" :aria-label="labels.page"><button :disabled="page===0" @click="move(-1)">{{labels.previous}}</button><label>{{labels.number}} <input type="number" min="1" :max="Math.ceil(results.length/pageSize)" :value="page+1" @change="jump"></label><span>{{page*pageSize+1}}–{{Math.min((page+1)*pageSize,results.length)}} / {{results.length}} {{labels.confirmed}}</span><button :disabled="(page+1)*pageSize>=results.length" @click="move(1)">{{labels.next}}</button></nav>
 <ol class="search-results" :start="page*pageSize+1"><li v-for="(result,index) in visible" :key="result.id" :data-result-id="result.id"><slot :result="result" :index="page*pageSize+index" /></li></ol>
</div>
</template>
