import type {Context,Limits,ProgramEntry} from './api';
import type {Locale} from './i18n';

export const workspaceKey='skating-c2-workspace';
export interface Workspace {
 version:1;context:Context;entries:ProgramEntry[];complete:boolean;personalLimits:Limits;
 excludedConnections:string[];allowMixedDirections:boolean;locale:Locale;
}
function record(value:unknown):value is Record<string,unknown>{return typeof value==='object'&&value!==null&&!Array.isArray(value);}
function exact(value:unknown,required:string[],optional:string[]=[]):value is Record<string,unknown>{return record(value)&&required.every(k=>k in value)&&Object.keys(value).every(k=>required.includes(k)||optional.includes(k));}
function nonempty(value:unknown):value is string{return typeof value==='string'&&value.length>0;}
function limit(value:unknown,min:number,max:number){return exact(value,['enabled'],['maximum'])&&typeof value.enabled==='boolean'&&(value.maximum===undefined||Number.isInteger(value.maximum)&&Number(value.maximum)>=min&&Number(value.maximum)<=max);}
function limitMap(value:unknown,min:number,max:number){return record(value)&&Object.entries(value).every(([key,v])=>nonempty(key)&&limit(v,min,max));}
export function parseWorkspace(raw:string):Workspace {
 const w:unknown=JSON.parse(raw);
 if(!exact(w,['version','context','entries','complete','personalLimits','excludedConnections','allowMixedDirections','locale'])||w.version!==1
 ||!exact(w.context,['profile','ruleEdition','scoringTable'])||!Object.values(w.context).every(nonempty)
 ||!Array.isArray(w.entries)||!w.entries.every(e=>exact(e,['id','elementKey','code','retained'])&&nonempty(e.id)&&nonempty(e.elementKey)&&nonempty(e.code)&&typeof e.retained==='boolean')
 ||new Set(w.entries.map(e=>e.id)).size!==w.entries.length||typeof w.complete!=='boolean'
 ||!exact(w.personalLimits,['jumps','spins','unlevelledElements'],['steps'])||!limitMap(w.personalLimits.jumps,1,5)||!limitMap(w.personalLimits.spins,0,4)
 ||(w.personalLimits.steps!==undefined&&!limit(w.personalLimits.steps,0,4))||!record(w.personalLimits.unlevelledElements)||!Object.values(w.personalLimits.unlevelledElements).every(v=>typeof v==='boolean')
 ||!Array.isArray(w.excludedConnections)||!w.excludedConnections.every(nonempty)||new Set(w.excludedConnections).size!==w.excludedConnections.length
 ||typeof w.allowMixedDirections!=='boolean'||!['en','de','ru'].includes(String(w.locale)))throw new Error('invalid_workspace');
 return w as unknown as Workspace;
}
// Explicit allowlist: never serialize application/search state or extra properties on adopted entries.
export function serializeWorkspace(w:Workspace):string {
 const limits=(map:Limits['jumps'])=>Object.fromEntries(Object.entries(map).map(([k,v])=>[k,{enabled:v.enabled,...(v.maximum===undefined?{}:{maximum:v.maximum})}]));
 return JSON.stringify({version:1,context:{profile:w.context.profile,ruleEdition:w.context.ruleEdition,scoringTable:w.context.scoringTable},entries:w.entries.map(({id,elementKey,code,retained})=>({id,elementKey,code,retained})),complete:w.complete,personalLimits:{jumps:limits(w.personalLimits.jumps),spins:limits(w.personalLimits.spins),...(w.personalLimits.steps?{steps:limits({steps:w.personalLimits.steps}).steps}:{}),unlevelledElements:Object.fromEntries(Object.entries(w.personalLimits.unlevelledElements).map(([k,v])=>[k,v]))},excludedConnections:[...w.excludedConnections],allowMixedDirections:w.allowMixedDirections,locale:w.locale});
}
export const workspaceText={
 en:{saved:'Workspace saved on this browser.',access:'Browser storage is unavailable. Changes are not saved.',save:'Workspace could not be saved. The previous snapshot is unchanged.',invalid:'Saved workspace is corrupt or uses an unsupported format. It has not been changed.',context:'Saved category, edition or program content is unavailable. The snapshot has not been changed.',retry:'Retry local workspace',replace:'Replace saved workspace with a new empty workspace'},
 de:{saved:'Arbeitsstand in diesem Browser gespeichert.',access:'Browserspeicher nicht verfügbar. Änderungen werden nicht gespeichert.',save:'Arbeitsstand nicht gespeichert. Der vorige Stand bleibt erhalten.',invalid:'Gespeicherter Arbeitsstand beschädigt oder Format nicht unterstützt. Er bleibt erhalten.',context:'Gespeicherte Kategorie, Version oder Programminhalt nicht verfügbar. Der Stand bleibt erhalten.',retry:'Lokalen Arbeitsstand erneut laden',replace:'Gespeicherten Stand durch einen neuen leeren Arbeitsstand ersetzen'},
 ru:{saved:'Рабочая область сохранена в этом браузере.',access:'Хранилище браузера недоступно. Изменения не сохраняются.',save:'Не удалось сохранить рабочую область. Предыдущая копия не изменена.',invalid:'Сохранённая рабочая область повреждена или формат не поддерживается. Копия не изменена.',context:'Сохранённая категория, версия или программа недоступна. Копия не изменена.',retry:'Повторить загрузку рабочей области',replace:'Заменить сохранённую область новой пустой областью'}
};
