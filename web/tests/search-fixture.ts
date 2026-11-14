import {test as base} from '@playwright/test';
export {expect} from '@playwright/test';

// Also release sessions whose creation response arrived after input invalidation.
export const test=base.extend<{searchCleanup:void}>({
 searchCleanup:[async({page},use)=>{
  const owners:Promise<{origin:string;id:string;accessToken:string}|undefined>[]=[];
  page.on('response',response=>{
   if(response.request().method()==='POST'&&new URL(response.url()).pathname==='/api/v1/searches'&&response.status()===202)
    owners.push(response.json().then(owner=>({origin:new URL(response.url()).origin,id:owner.id,accessToken:owner.accessToken})).catch(()=>undefined));
  });
  await use();
  for(const owner of await Promise.all(owners))if(owner)await page.request.delete(`${owner.origin}/api/v1/searches/${owner.id}`,{headers:{Authorization:`Bearer ${owner.accessToken}`}});
 },{auto:true}]
});
