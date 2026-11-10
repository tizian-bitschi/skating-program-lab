import {test,expect} from '@playwright/test';
// Each test owns and releases the real native sessions it creates, including
// responses whose browser state has already been invalidated.
const searchOwners=new WeakMap<object,Promise<{origin:string;id:string;accessToken:string}|undefined>[]>();
test.beforeEach(async({page})=>{
 const owners:Promise<{origin:string;id:string;accessToken:string}|undefined>[]=[];searchOwners.set(page,owners);
 page.on('response',response=>{if(response.request().method()==='POST'&&new URL(response.url()).pathname==='/api/v1/searches'&&response.status()===202)owners.push(response.json().then(owner=>({origin:new URL(response.url()).origin,id:owner.id,accessToken:owner.accessToken})).catch(()=>undefined));});
});
test.afterEach(async({page})=>{
 for(const owner of await Promise.all(searchOwners.get(page)??[]))if(owner)await page.request.delete(`${owner.origin}/api/v1/searches/${owner.id}`,{headers:{Authorization:`Bearer ${owner.accessToken}`}});
});
test('explicit proof adopts a freshly assessed program and preserves retained identity',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 let creates=0;page.on('request',r=>{if(r.url().endsWith('/api/v1/searches')&&r.method()==='POST')creates++;});
 for(const code of ['CCSp1','CLSp1','ChSq1']){await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();await page.getByTestId('program-entry').last().getByRole('checkbox').check();}
 const ids=await page.getByTestId('program-entry').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-entry-id')));
 for(const kind of ['T','S','Lz'])await page.getByTestId(`enabled-${kind}`).uncheck();await page.getByTestId('enabled-W').uncheck();
 expect(creates).toBe(0);await page.getByTestId('calculate').focus();await page.keyboard.press('Enter');
 await expect(page.getByTestId('search-maximum')).toContainText('10.30',{timeout:30000});await expect(page.getByTestId('adopt-result').first()).toBeEnabled();
 await page.getByTestId('adopt-result').first().focus();await page.keyboard.press('Enter');await expect(page.getByTestId('assessment')).toContainText('Construction complies');await expect(page.getByTestId('assessment')).toContainText('10.30');
 const adopted=page.getByTestId('program-entry');for(const id of ids){await expect(page.locator(`[data-entry-id="${id}"]`).getByRole('checkbox')).toBeChecked();}
 expect(await adopted.count()).toBeGreaterThan(3);expect(await adopted.locator('input:checked').count()).toBe(3);
 const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!));expect(saved.entries.filter((e:{retained:boolean})=>e.retained).map((e:{id:string})=>e.id).sort()).toEqual([...ids].sort());expect(saved.entries.some((e:{retained:boolean})=>!e.retained)).toBe(true);expect(JSON.stringify(saved)).not.toMatch(/accessToken|creationKey|confirmedCount/);

 await page.getByLabel('Language').selectOption('de');await page.getByLabel('Language').selectOption('ru');await page.getByLabel('Language').selectOption('en');await adopted.last().getByRole('button',{name:'Move up'}).click();expect(creates).toBe(1);await expect(page.getByTestId('search-stale')).toHaveCount(0);
 await page.getByTestId('maximum-F').selectOption('1');await expect(page.getByTestId('search-stale')).toBeVisible();await expect(page.getByTestId('adopt-result').first()).toBeDisabled();expect(creates).toBe(1);
 await page.reload();await expect(page.getByTestId('assessment')).toContainText('Construction complies');expect(creates).toBe(1);await expect(page.getByTestId('program-entry')).toHaveCount(saved.entries.length);for(const id of ids)await expect(page.locator(`[data-entry-id="${id}"]`).getByRole('checkbox')).toBeChecked();await expect(page.getByTestId('adopt-result')).toHaveCount(0);
});


test('proved adoption uses the six real profiles and an independently registered policy',async({page})=>{
 test.setTimeout(90000);
 for(const origin of ['',`http://127.0.0.1:${process.env.C2_FIXTURE_PORT??3158}`]){
 await page.goto(origin||'/');const profiles=await(await page.request.get(`${origin}/api/v1/profiles`)).json();
 const bases:Record<string,string[]>={bronze:['CCSp1','CLSp1','ChSq1'],silver:['1A','CLSp2','ChSp1','ChSq1'],gold:['1A','CCoSp3','FSSp3','ChSp1','StSq2'],masters:['1A','CoSp4','FSSp4','ChSp1','StSq4'],women:['1A','CoSp4','FSSp4','ChSp1','StSq4','ChSq1'],men:['1A','CoSp4','FSSp4','ChSp1','StSq4','ChSq1'],'Independent HTTP test policy':['1Lz','SSp1','ChSq1']};
 for(const p of profiles.filter((p:{label:string})=>origin?p.label==='Independent HTTP test policy':true)){
 const codes=bases[p.label]!;const response=await page.request.post(`${origin}/api/v1/repertoire`,{data:{requestId:'fixture',revision:0,context:p.context,personalLimits:{}}});const repertoire=await response.json();
 const selected=codes.map(code=>{const e=repertoire.elements.find((e:{code:string})=>e.code===code);expect(e,code).toBeTruthy();return e;});
 const jumps=Object.fromEntries(repertoire.forms.filter((f:{family:string})=>f.family==='jump').map((f:{id:string})=>[f.id,{enabled:['A','Lz'].includes(f.id),maximum:1}]));
 const spins=Object.fromEntries(repertoire.forms.filter((f:{family:string})=>f.family==='spin').map((f:{id:string})=>{const chosen=selected.find(e=>e.form===f.id);return[f.id,{enabled:!!chosen,...(chosen?{maximum:chosen.level}:{})}]}));
 const step=selected.find(e=>e.family==='step');const unlevelledElements=Object.fromEntries(repertoire.forms.filter((f:{family:string})=>f.family==='unlevelled').map((f:{id:string})=>[f.id,selected.some(e=>e.form===f.id)]));
 const workspace={version:1,context:p.context,entries:selected.map((e,i)=>({id:`fixture-${i}`,elementKey:e.key,code:e.code,retained:true})),complete:true,personalLimits:{jumps,spins,steps:{enabled:!!step,...(step?{maximum:step.level}:{})},unlevelledElements},excludedConnections:[],allowMixedDirections:true,locale:'en'};
 await page.evaluate(w=>localStorage.setItem('skating-c2-workspace',JSON.stringify(w)),workspace);await page.reload();await expect(page.getByTestId('assessment')).toContainText('Construction complies');const start=Date.now();await page.getByTestId('calculate').click();await expect(page.getByTestId('adopt-result').first()).toBeEnabled({timeout:30000});console.log(`proved/adoptable ${p.label}: ${Date.now()-start}ms`);await page.getByTestId('adopt-result').first().click();await expect(page.getByTestId('assessment')).toContainText('Construction complies');for(let i=0;i<selected.length;i++)await expect(page.locator(`[data-entry-id="fixture-${i}"]`).getByRole('checkbox')).toBeChecked();
 }
 }
});

test('semantic edits during namespace or creation waits cannot start or adopt stale work',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 let creates=0;page.on('request',r=>{if(r.method()==='POST'&&r.url().endsWith('/api/v1/searches'))creates++;});
 let releaseNamespace=()=>{};const namespaceHeld=new Promise<void>(r=>releaseNamespace=r);let namespaceSeen=()=>{};const seenNamespace=new Promise<void>(r=>namespaceSeen=r);
 await page.route('**/api/v1/searches/identity',async route=>{const response=await route.fetch();namespaceSeen();await namespaceHeld;await route.fulfill({response});});
 await page.getByTestId('calculate').click();await seenNamespace;await page.getByTestId('maximum-F').selectOption('1');releaseNamespace();await expect(page.getByTestId('calculate')).toBeEnabled();await page.unroute('**/api/v1/searches/identity');expect(creates).toBe(0);
 let releaseCreation=()=>{};const creationHeld=new Promise<void>(r=>releaseCreation=r);let creationSeen=()=>{};const seenCreation=new Promise<void>(r=>creationSeen=r);let owner:{id:string;accessToken:string}|undefined;
 await page.route('**/api/v1/searches',async route=>{const response=await route.fetch();owner=await response.json();creationSeen();await creationHeld;await route.fulfill({response});});
 await page.getByTestId('calculate').click();await seenCreation;await page.getByTestId('maximum-Lo').selectOption('1');releaseCreation();await expect(page.getByTestId('calculate')).toBeEnabled();await expect.poll(async()=>{const response=await page.request.get(`/api/v1/searches/${owner!.id}/updates`,{headers:{Authorization:`Bearer ${owner!.accessToken}`}});return response.status();}).toBe(404);expect(creates).toBe(1);await expect(page.getByTestId('adopt-result')).toHaveCount(0);await expect(page.getByTestId('search-maximum')).toHaveCount(0);
});


test('real collection pauses, continues, compares and adopts without losing focus',async({page})=>{
 test.setTimeout(90000);
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await page.getByLabel('Category').selectOption('isu-adult-2026-27-silver');for(const kind of ['T','S','Lz'])await page.getByTestId(`enabled-${kind}`).uncheck();for(const kind of ['F','Lo','A'])await page.getByTestId(`maximum-${kind}`).selectOption('1');await page.getByTestId('enabled-W').uncheck();await page.getByTestId('calculate').click();
 const control=page.getByTestId('collection-control');await expect(control).toHaveText('Pause collection',{timeout:30000});await control.focus();await page.keyboard.press('Enter');await expect(control).toHaveText('Continue collection',{timeout:30000});await expect(control).toBeFocused();
 await expect(page.getByTestId('adopt-result').first()).toBeEnabled();const compare=page.locator('.search-results input[type=checkbox]');await compare.nth(0).check();await compare.nth(1).check();await expect(page.getByTestId('program-comparison').locator('tbody tr')).toHaveCount(2);
 const before=await page.locator('.search-results > li').first().getAttribute('data-result-id');
 for(const [locale,label] of [['de','Sammlung fortsetzen'],['ru','Продолжить сбор'],['en','Continue collection']]){await page.getByLabel('Language').selectOption(locale);await expect(control).toHaveText(label);}
 await control.focus();await page.keyboard.press('Enter');await expect(control).toHaveText('Pause collection',{timeout:10000});await expect(control).toBeFocused();await expect(page.locator('.search-results > li').first()).toHaveAttribute('data-result-id',before!);
 let lost=true;const commands:string[]=[];await page.route('**/api/v1/searches/*/pause',async route=>{commands.push(route.request().postData()!);const response=await route.fetch();if(lost){lost=false;await route.abort('failed');}else await route.fulfill({response});});
 await control.click();await page.getByRole('button',{name:'Retry connection',exact:true}).click();await expect(control).toHaveText('Continue collection',{timeout:10000});expect(commands).toHaveLength(2);expect(commands[0]).toBe(commands[1]);
 await page.getByRole('button',{name:'Cancel calculation',exact:true}).click();await expect(page.getByText('Calculation cancelled.',{exact:true})).toBeVisible({timeout:10000});await page.getByTestId('adopt-result').first().focus();await page.keyboard.press('Enter');await expect(page.getByTestId('assessment')).toContainText('Construction complies');
});
