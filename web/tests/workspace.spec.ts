import {openPicker,setMaximum,setEnabled} from './picker';
import {test,expect} from '@playwright/test';
test('workspace reload preserves entries, retention, limits and locale without a search',async({page})=>{
 let searches=0;page.on('request',r=>{if(r.url().includes('/api/v1/searches'))searches++;});
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await openPicker(page);await page.getByTestId('element-query').fill('1Lz');await page.getByTestId('add-1Lz').click();await page.getByTestId('program-entry').getByRole('checkbox').check();const id=await page.getByTestId('program-entry').getAttribute('data-entry-id');await setMaximum(page,'T',1);await page.getByLabel('Language').selectOption('de');const restoredAt=Date.now();await page.reload();await expect(page.getByTestId('program-entry')).toHaveAttribute('data-entry-id',id!);await expect(page.getByTestId('program-entry').getByRole('checkbox')).toBeChecked();await expect(page.getByLabel('Language')).toHaveValue('de');await expect(page.getByTestId('assessment')).toContainText('0,60');console.log(`workspace reload to matching assessment: ${Date.now()-restoredAt}ms; ${await page.evaluate(()=>navigator.userAgent)}`);expect(searches).toBe(0);
});

test('corrupt snapshot stays intact until explicit replacement',async({page})=>{
 await page.addInitScript(()=>localStorage.setItem('skating-c2-workspace','{broken'));
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await expect(page.getByTestId('workspace-storage')).toContainText('corrupt');expect(await page.evaluate(()=>localStorage.getItem('skating-c2-workspace'))).toBe('{broken');await page.getByRole('button',{name:'Replace saved workspace with a new empty workspace',exact:true}).focus();await page.keyboard.press('Enter');await expect(page.getByTestId('workspace-storage')).toContainText('Workspace saved');await expect(page.getByTestId('program-entry')).toHaveCount(0);await expect(page.getByLabel('Category')).toBeFocused();
});

test('unsupported schema and missing context never fall back or overwrite',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await expect(page.getByTestId('workspace-storage')).toContainText('saved');
 const good=await page.evaluate(()=>localStorage.getItem('skating-c2-workspace')!);
 for(const invalid of [{...JSON.parse(good),version:999},{...JSON.parse(good),context:{profile:'missing',ruleEdition:'missing',scoringTable:'missing'}},{...JSON.parse(good),entries:[{id:'one',elementKey:'missing:key',code:'1Lz',retained:true}]}]){
 const raw=JSON.stringify(invalid);await page.evaluate(raw=>localStorage.setItem('skating-c2-workspace',raw),raw);await page.reload();await expect(page.getByTestId('workspace-storage')).toContainText(/unsupported|unavailable/);await expect(page.getByLabel('Category')).toHaveValue('');expect(await page.evaluate(()=>localStorage.getItem('skating-c2-workspace'))).toBe(raw);
 }
});

test('quota failure preserves previous save and explicit retry saves current edits',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await expect(page.getByTestId('workspace-storage')).toContainText('saved');
 const before=await page.evaluate(()=>localStorage.getItem('skating-c2-workspace'));
 await page.evaluate(()=>{const original=Storage.prototype.setItem;Object.assign(window,{restoreStorage:()=>{Storage.prototype.setItem=original;}});Storage.prototype.setItem=function(){throw new DOMException('Quota exceeded','QuotaExceededError');};});
 await openPicker(page);await page.getByTestId('element-query').fill('1Lz');await page.getByTestId('add-1Lz').click();await expect(page.getByTestId('workspace-storage')).toContainText('could not be saved');expect(await page.evaluate(()=>localStorage.getItem('skating-c2-workspace'))).toBe(before);
 await page.evaluate(()=>(window as unknown as {restoreStorage:()=>void}).restoreStorage());await page.getByRole('button',{name:'Retry local workspace'}).click();await expect(page.getByTestId('workspace-storage')).toContainText('saved on this browser');await page.reload();await expect(page.getByTestId('program-entry')).toContainText('1Lz');
});

test('inaccessible storage reports failure and retains its snapshot',async({page})=>{
 await page.addInitScript(()=>{localStorage.setItem('skating-c2-workspace','original snapshot');Storage.prototype.getItem=function(){throw new DOMException('Blocked','SecurityError');};});await page.goto('/');await page.getByLabel('Language').selectOption('en');await expect(page.getByTestId('workspace-storage')).toContainText('storage is unavailable');expect(await page.evaluate(()=>localStorage['skating-c2-workspace'])).toBe('original snapshot');
});

test('saved locale wins over browser locale; unsupported browser falls back to English',async({browser})=>{
 for(const [language,expected] of [['ru-RU','ru'],['fr-FR','en']]){
 const context=await browser.newContext({locale:language});const page=await context.newPage();await page.goto(`http://127.0.0.1:${process.env.C2_TEST_PORT??3161}`);await expect(page.getByLabel('Language')).toHaveValue(expected);await page.getByLabel('Language').selectOption('de');await page.reload();await expect(page.getByLabel('Language')).toHaveValue('de');await context.close();
 }
});

test('snapshot allowlist contains only bounded workspace inputs',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await openPicker(page);await page.getByTestId('element-query').fill('1Lz');await page.getByTestId('add-1Lz').click();await page.getByText('Program is fully described',{exact:true}).click();await setEnabled(page,'T',false);await page.reload();await expect(page.getByTestId('fader-T')).toHaveValue('0');await expect(page.getByText('Program is fully described',{exact:true}).getByRole('checkbox')).toBeChecked();
 const snapshot=await page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!));expect(Object.keys(snapshot).sort()).toEqual(['allowMixedDirections','complete','context','entries','excludedConnections','locale','personalLimits','version']);expect(Object.keys(snapshot.entries[0]).sort()).toEqual(['code','elementKey','id','retained']);expect(snapshot.excludedConnections).toEqual([]);expect(snapshot.allowMixedDirections).toBe(false);expect(snapshot.personalLimits.jumps.T.enabled).toBe(false);
});

test('exact connection exclusions and rotation preference survive category changes',async({page})=>{
 let latest:Record<string,unknown>={};page.on('request',r=>{if(r.url().endsWith('/program-assessments')&&r.postDataJSON().revision>0)latest=r.postDataJSON();});
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await expect(page.getByTestId('workspace-storage')).toContainText('saved');const snapshot=await page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!));
 const profiles=await(await page.request.get('/api/v1/profiles')).json();const silver=profiles.find((p:{label:string})=>p.label==='silver');
 const response=await page.request.post('/api/v1/connections/options',{data:{requestId:'workspace-test',revision:0,context:silver.context,personalLimits:{jumps:{},spins:{},unlevelledElements:{}},prefix:['1Lz','1Lo']}});expect(response.ok()).toBe(true);const pair=(await response.json()).completions.find((e:{code:string})=>e.code==='1Lz+1Lo');expect(pair).toBeTruthy();
 snapshot.context=silver.context;snapshot.entries=[{id:'adopted-occurrence',elementKey:pair.key,code:pair.code,retained:true}];snapshot.excludedConnections=[pair.key];snapshot.allowMixedDirections=true;snapshot.complete=true;
 await page.evaluate(w=>localStorage.setItem('skating-c2-workspace',JSON.stringify(w)),snapshot);await page.reload();await expect(page.getByTestId('program-entry')).toHaveAttribute('data-entry-id','adopted-occurrence');await expect(page.getByTestId('program-entry').getByRole('checkbox')).toBeChecked();await page.getByLabel('Category').selectOption({label:'Adult Gold'});await page.reload();await expect(page.getByLabel('Category')).toHaveValue(profiles.find((p:{label:string})=>p.label==='gold').context.profile);
 const restored=await page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!));expect(restored.excludedConnections).toEqual([pair.key]);expect(restored.allowMixedDirections).toBe(true);expect(restored.entries).toEqual(snapshot.entries);await expect.poll(()=>latest.allowMixedDirections).toBe(true);expect(latest.excludedConnections).toEqual([pair.key]);
});

test('storage error recovery stays readable in all languages and narrow layouts',async({page})=>{
 await page.addInitScript(()=>localStorage.setItem('skating-c2-workspace','unsupported'));
 await page.goto('/');
 for(const locale of ['de','en','ru']){await page.getByLabel('Language').selectOption(locale);for(const width of [1280,390,320]){await page.setViewportSize({width,height:900});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await expect(page.getByTestId('workspace-storage').getByRole('button').last()).toBeVisible();}}
});
