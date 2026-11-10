import {openPicker,setMaximum,setEnabled} from './picker';
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
 for(const code of ['CCSp1','CLSp1','ChSq1']){await openPicker(page);await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();await page.getByTestId('program-entry').last().getByRole('checkbox').check();}
 const ids=await page.getByTestId('program-entry').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-entry-id')));
 for(const kind of ['T','S','Lz'])await setEnabled(page,`${kind}`,false);await setEnabled(page,'W',false);
 expect(creates).toBe(0);await page.getByTestId('calculate').focus();await page.keyboard.press('Enter');
 await expect(page.getByTestId('search-maximum')).toContainText('10.30',{timeout:30000});await expect(page.getByTestId('adopt-result').first()).toBeEnabled();
 await page.getByTestId('adopt-result').first().focus();await page.keyboard.press('Enter');await expect(page.getByTestId('assessment')).toContainText('Construction complies');await expect(page.getByTestId('assessment')).toContainText('10.30');
 const adopted=page.getByTestId('program-entry');for(const id of ids){await expect(page.locator(`[data-entry-id="${id}"]`).getByRole('checkbox')).toBeChecked();}
 expect(await adopted.count()).toBeGreaterThan(3);expect(await adopted.locator('input:checked').count()).toBe(3);
 const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!));expect(saved.entries.filter((e:{retained:boolean})=>e.retained).map((e:{id:string})=>e.id).sort()).toEqual([...ids].sort());expect(saved.entries.some((e:{retained:boolean})=>!e.retained)).toBe(true);expect(JSON.stringify(saved)).not.toMatch(/accessToken|creationKey|confirmedCount/);

 await page.getByLabel('Language').selectOption('de');await page.getByLabel('Language').selectOption('ru');await page.getByLabel('Language').selectOption('en');await adopted.last().getByRole('button',{name:'Move up'}).click();expect(creates).toBe(1);await expect(page.getByTestId('search-stale')).toHaveCount(0);
 await setMaximum(page,'F',1);await expect(page.getByTestId('search-stale')).toBeVisible();await expect(page.getByTestId('adopt-result').first()).toBeDisabled();expect(creates).toBe(1);
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
 await page.getByTestId('calculate').click();await seenNamespace;await setMaximum(page,'F',1);releaseNamespace();await expect(page.getByTestId('calculate')).toBeEnabled();await page.unroute('**/api/v1/searches/identity');expect(creates).toBe(0);
 let releaseCreation=()=>{};const creationHeld=new Promise<void>(r=>releaseCreation=r);let creationSeen=()=>{};const seenCreation=new Promise<void>(r=>creationSeen=r);let owner:{id:string;accessToken:string}|undefined;
 await page.route('**/api/v1/searches',async route=>{const response=await route.fetch();owner=await response.json();creationSeen();await creationHeld;await route.fulfill({response});});
 await page.getByTestId('calculate').click();await seenCreation;await setMaximum(page,'Lo',1);releaseCreation();await expect(page.getByTestId('calculate')).toBeEnabled();await expect.poll(async()=>{const response=await page.request.get(`/api/v1/searches/${owner!.id}/updates`,{headers:{Authorization:`Bearer ${owner!.accessToken}`}});return response.status();}).toBe(404);expect(creates).toBe(1);await expect(page.getByTestId('adopt-result')).toHaveCount(0);await expect(page.getByTestId('search-maximum')).toHaveCount(0);
});


test('real collection pauses, continues, compares and adopts without losing focus',async({page})=>{
 test.setTimeout(90000);
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await page.getByLabel('Category').selectOption('isu-adult-2026-27-silver');for(const kind of ['T','S','Lz'])await setEnabled(page,`${kind}`,false);for(const kind of ['F','Lo','A'])await setMaximum(page,`${kind}`,1);await setEnabled(page,'W',false);await page.getByTestId('calculate').click();
 const control=page.getByTestId('collection-control');await expect(control).toHaveText('Pause collection',{timeout:30000});await control.focus();await page.keyboard.press('Enter');await expect(control).toHaveText('Continue collection',{timeout:30000});await expect(control).toBeFocused();
 let disconnected=false;const reconnectReads:string[]=[];
 await page.route('**/api/v1/searches/*/updates?*',async route=>{reconnectReads.push(route.request().url());if(!disconnected){disconnected=true;await route.abort('failed');}else await route.continue();});
 await page.getByRole('button',{name:'Retry connection',exact:true}).click();await expect(control).toHaveText('Continue collection');await expect.poll(()=>reconnectReads.length).toBeGreaterThanOrEqual(2);expect(reconnectReads[0]).toBe(reconnectReads[1]);await page.unroute('**/api/v1/searches/*/updates?*');
 await expect(page.getByTestId('adopt-result').first()).toBeEnabled();const compare=page.locator('.search-results input[type=checkbox]');await compare.nth(0).check();await compare.nth(1).check();await expect(page.getByTestId('program-comparison').locator('tbody tr')).toHaveCount(2);
 const before=await page.locator('.search-results > li').first().getAttribute('data-result-id');
 for(const [locale,label] of [['de','Sammlung fortsetzen'],['ru','Продолжить сбор'],['en','Continue collection']]){await page.getByLabel('Language').selectOption(locale);await expect(control).toHaveText(label);}
 await control.focus();await page.keyboard.press('Enter');await expect(control).toHaveText('Pause collection',{timeout:10000});await expect(control).toBeFocused();await expect(page.locator('.search-results > li').first()).toHaveAttribute('data-result-id',before!);
 let releaseOld=()=>{},seenOld=()=>{};const oldHeld=new Promise<void>(r=>releaseOld=r),oldSeen=new Promise<void>(r=>seenOld=r);let holdOld=true;
 await page.route('**/api/v1/searches/*/updates?*',async route=>{const response=await route.fetch();if(holdOld){holdOld=false;seenOld();await oldHeld;}await route.fulfill({response});});await oldSeen;
 let lost=true;const commands:string[]=[];await page.route('**/api/v1/searches/*/pause',async route=>{commands.push(route.request().postData()!);const response=await route.fetch();if(lost){lost=false;await route.abort('failed');}else await route.fulfill({response});});
 await control.click();await page.getByRole('button',{name:'Retry connection',exact:true}).click();await expect(control).toHaveText('Continue collection',{timeout:10000});expect(commands).toHaveLength(2);expect(commands[0]).toBe(commands[1]);releaseOld();await page.unroute('**/api/v1/searches/*/updates?*');await expect(control).toHaveText('Continue collection');
 const continueCommands:string[]=[];await page.route('**/api/v1/searches/*/continue',async route=>{continueCommands.push(route.request().postData()!);const response=await route.fetch();if(continueCommands.length===1)await route.abort('failed');else await route.fulfill({response});});
 await control.click();await page.getByRole('button',{name:'Retry connection',exact:true}).click();await expect(control).toHaveText('Pause collection');expect(continueCommands).toHaveLength(2);expect(continueCommands[0]).toBe(continueCommands[1]);
 const cancelCommands:string[]=[];await page.route('**/api/v1/searches/*/cancel',async route=>{cancelCommands.push(route.request().postData()!);const response=await route.fetch();if(cancelCommands.length===1)await route.abort('failed');else await route.fulfill({response});});
 await page.getByRole('button',{name:'Cancel calculation',exact:true}).click();await page.getByRole('button',{name:'Retry connection',exact:true}).click();expect(cancelCommands).toHaveLength(2);expect(cancelCommands[0]).toBe(cancelCommands[1]);await expect(page.getByText('Calculation cancelled.',{exact:true})).toBeVisible({timeout:10000});await page.getByTestId('adopt-result').first().focus();await page.keyboard.press('Enter');await expect(page.getByTestId('assessment')).toContainText('Construction complies');
});

test('lost accepted creation retries its capability and expired sessions require explicit calculation',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 await page.getByLabel('Category').selectOption('isu-adult-2026-27-silver');for(const kind of ['T','S','Lz'])await setEnabled(page,`${kind}`,false);for(const kind of ['F','Lo','A'])await setMaximum(page,`${kind}`,1);await setEnabled(page,'W',false);
 const bodies:string[]=[];let owner:{id:string;accessToken:string}|undefined;
 await page.route('**/api/v1/searches',async route=>{bodies.push(route.request().postData()!);const response=await route.fetch();owner=await response.json();if(bodies.length===1)await route.abort('failed');else await route.fulfill({response});});
 await page.getByTestId('calculate').click();await page.getByRole('button',{name:'Retry connection',exact:true}).click();await expect.poll(()=>bodies.length).toBe(2);expect(bodies[0]).toBe(bodies[1]);await expect(page.getByTestId('search-maximum')).toBeVisible({timeout:30000});await expect(page.getByTestId('adopt-result').first()).toBeEnabled();const maximum=await page.getByTestId('search-maximum').textContent();const first=await page.locator('[data-result-id]').first().getAttribute('data-result-id');
 await page.request.delete(`/api/v1/searches/${owner!.id}`,{headers:{Authorization:`Bearer ${owner!.accessToken}`}});
 await expect(page.getByRole('alert')).toContainText('This session is unavailable.',{timeout:10000});
 await expect(page.getByRole('button',{name:'Retry connection',exact:true})).toHaveCount(0);for(const [locale,message] of [['de','Diese Sitzung ist nicht verfügbar.'],['ru','Сессия недоступна.'],['en','This session is unavailable.']]){await page.getByLabel('Language').selectOption(locale);await expect(page.getByRole('alert')).toContainText(message);}
 await expect(page.getByTestId('calculate')).toBeEnabled();expect(bodies).toHaveLength(2);await expect(page.getByTestId('search-maximum')).toHaveText(maximum!);await expect(page.locator('[data-result-id]').first()).toHaveAttribute('data-result-id',first!);await expect(page.getByTestId('adopt-result').first()).toBeEnabled();
});


test('proof disconnection reconnects the page-held identity without another calculation',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 for(const code of ['CCSp1','CLSp1','ChSq1']){await openPicker(page);await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();await page.getByTestId('program-entry').last().getByRole('checkbox').check();}
 for(const kind of ['T','S','Lz'])await setEnabled(page,`${kind}`,false);await setEnabled(page,'W',false);
 let creates=0;page.on('request',r=>{if(r.method()==='POST'&&r.url().endsWith('/api/v1/searches'))creates++;});
 const reads:string[]=[];await page.route('**/api/v1/searches/*/updates?*',async route=>{reads.push(route.request().url());const response=await route.fetch();if(reads.length===1)await route.abort('failed');else await route.fulfill({response});});
 const accepted=page.waitForResponse(r=>r.request().method()==='POST'&&r.url().endsWith('/api/v1/searches'));await page.getByTestId('calculate').click();const owner=await(await accepted).json();await page.getByRole('button',{name:'Retry connection',exact:true}).click();await expect(page.getByTestId('search-maximum')).toContainText('10.30',{timeout:30000});expect(creates).toBe(1);expect(reads[0]).toBe(reads[1]);
 const ids=await page.locator('[data-result-id]').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-result-id')));expect(new Set(ids).size).toBe(ids.length);
 await page.reload();await expect(page.getByTestId('calculate')).toBeEnabled();await expect(page.getByTestId('search-maximum')).toHaveCount(0);expect(creates).toBe(1);
 await expect.poll(async()=>{const response=await page.request.get(`/api/v1/searches/${owner.id}/updates`,{headers:{Authorization:`Bearer ${owner.accessToken}`}});return response.status();}).toBe(404);
});

test('called Waltz conflict names Axel Off and recovers when Axel is enabled',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await page.getByLabel('Category').selectOption('isu-adult-2026-27-silver');
 const repertoire=await(await page.request.post('/api/v1/repertoire',{data:{requestId:'waltz-browser',revision:0,context:(await(await page.request.get('/api/v1/profiles')).json())[1].context,personalLimits:{}}})).json();
 const w=repertoire.elements.find((e:{form:string})=>e.form==='W');expect(w).toBeTruthy();
 await openPicker(page);await page.getByTestId('element-query').fill('W');await page.getByTestId(`add-${w.code}`).click();
 const conflict=page.waitForResponse(async r=>r.url().endsWith('/api/v1/program-assessments')&&r.request().method()==='POST'&&(await r.json()).personalConflicts?.some((f:{setting:string[]})=>f.setting?.includes('A')));
 await setEnabled(page,'A',false);const assessed=await(await conflict).json();expect(assessed.personalConflicts[0].setting).toEqual(['A']);
 await openPicker(page);await page.getByTestId('element-query').fill('W');await expect(page.getByTestId(`add-${w.code}`)).toHaveCount(0);await page.keyboard.press('Escape');
 const restored=page.waitForResponse(async r=>r.url().endsWith('/api/v1/program-assessments')&&r.request().method()==='POST'&&(await r.json()).personalConflicts?.length===0);
 await setEnabled(page,'A',true);await restored;
 await openPicker(page);await page.getByTestId('element-query').fill('W');await expect(page.getByTestId(`add-${w.code}`)).toBeVisible();
});
