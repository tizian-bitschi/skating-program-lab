import type {ProgramEntry} from './api';

// Anchor restored occurrences to surviving neighbours, keeping later edits intact.
export function restoreOccurrences(current:ProgramEntry[],before:ProgramEntry[],removed:ProgramEntry[]){
 const restored=current.map(entry=>({...entry}));
 for(const entry of removed){
  if(restored.some(item=>item.id===entry.id))continue;
  const index=before.findIndex(item=>item.id===entry.id);
  const following=before.slice(index+1).find(item=>restored.some(saved=>saved.id===item.id));
  const preceding=before.slice(0,index).reverse().find(item=>restored.some(saved=>saved.id===item.id));
  const position=following?restored.findIndex(item=>item.id===following.id):preceding?restored.findIndex(item=>item.id===preceding.id)+1:Math.min(index,restored.length);
  restored.splice(position,0,{...entry});
 }
 return restored;
}
