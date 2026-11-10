export interface Context { profile:string; ruleEdition:string; scoringTable:string }
export interface Source { document:string; version:string; location:string; url?:string }
export interface Profile { context:Context; label:string; translationKey:string; sources:Source[] }
export interface Limit { enabled:boolean; maximum?:number }
export interface Limits { jumps:Record<string,Limit>;spins:Record<string,Limit>;steps?:Limit;unlevelledElements:Record<string,boolean> }
export interface Form { id:string;family:'jump'|'spin'|'step'|'unlevelled';available:boolean;eligibleLevels:number[];permittedLevels:number[] }
export interface Element {key:string;code:string;form:string;family:string;level:number|null;baseHundredths:number;source:Source}
export interface Repertoire {requestId:string;revision:number;context:Context;forms:Form[];elements:Element[]}
export const emptyLimits=():Limits=>({jumps:{},spins:{},unlevelledElements:{}});
export async function fetchJson<T>(path:string,body?:unknown):Promise<T>{
 const response=await fetch(path,body===undefined?undefined:{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});
 if(!response.ok)throw new Error((await response.json().catch(()=>({code:'connection_failure'}))).code);
 return response.json();
}

export interface ProgramEntry {id:string;elementKey:string;code:string;retained:boolean}
