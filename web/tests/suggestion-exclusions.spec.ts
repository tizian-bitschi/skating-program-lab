import {test,expect,type Page} from '@playwright/test';
import {configureSilver,openPicker} from './picker';
import {mkdir} from 'node:fs/promises';

const owners=new WeakMap<Page,Promise<{id:string;accessToken:string}>[]>();
test.beforeEach(({page})=>{
 const sessions:Promise<{id:string;accessToken:string}>[]=[];owners.set(page,sessions);
 page.on('response',r=>{if(new URL(r.url()).pathname==='/api/v1/searches'&&r.request().method()==='POST'&&r.status()===202)sessions.push(r.json());});
});
test.afterEach(async({page})=>{for(const owner of await Promise.all(owners.get(page)??[]))await page.request.delete(`/api/v1/searches/${owner.id}`,{headers:{Authorization:`Bearer ${owner.accessToken}`}});});

async function workspace(page:Page){return page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!));}
async function start(page:Page){await page.goto('/');await page.getByLabel('Language').selectOption('en');await configureSilver(page);}
const cards=(page:Page)=>page.locator('.planning-search>.result-browser');
async function connection(page:Page,code:string){
 const list=cards(page);
 for(let i=0;i<20;i++){
  const action=list.getByRole('button',{name:`Exclude connection ${code}`,exact:true}).first();
  if(await action.count())return action;
  const next=list.getByRole('button',{name:'Next programs',exact:true});
  await expect(next).toBeEnabled();await next.click();
 }
 throw new Error(`Connection absent: ${code}`);
}

test('collect pair and sequence exclusions from old cards and explicitly refresh the real Silver search',async({page})=>{
 test.setTimeout(120000);await start(page);
 let creates=0;const inputs:{excludedConnections:string[]}[]=[];
 page.on('request',r=>{if(new URL(r.url()).pathname==='/api/v1/searches'&&r.method()==='POST'){creates++;inputs.push(r.postDataJSON());}});
 await page.getByTestId('calculate').click();await expect(page.locator('.planning-search')).toContainText('All optimal compositions collected.',{timeout:45000});
 await (await connection(page,'1Lz+1A+1A+SEQ')).click();
 await expect(page.getByTestId('search-stale')).toBeVisible();
 await expect(cards(page).getByTestId('adopt-result').first()).toBeDisabled();
 await (await connection(page,'1Lz+1Lo')).focus();await page.keyboard.press('Enter');
 expect(creates).toBe(1);const excluded=(await workspace(page)).excludedConnections;expect(excluded).toHaveLength(2);
 expect(excluded.some((key:string)=>key.endsWith(':1Lz+1A+1A+SEQ:true'))).toBe(true);
 await page.getByRole('button',{name:'Update suggestions',exact:true}).click();
 await expect(page.locator('.planning-search')).toContainText('All optimal compositions collected.',{timeout:45000});
 expect(creates).toBe(2);expect(inputs[1].excludedConnections).toEqual(excluded);
 do{
  await expect(cards(page).getByRole('button',{name:'Exclude connection 1Lz+1Lo',exact:true})).toHaveCount(0);
  await expect(cards(page).getByRole('button',{name:'Exclude connection 1Lz+1A+1A+SEQ',exact:true})).toHaveCount(0);
  const next=cards(page).getByRole('button',{name:'Next programs',exact:true});if(!await next.count()||await next.isDisabled())break;await next.click();
 }while(true);
 await openPicker(page);await page.getByTestId('element-query').fill('1Lz');await expect(page.getByTestId('add-1Lz')).toBeVisible();await page.getByTestId('close-element-picker').click();
 const manager=page.getByTestId('connection-exclusions');await manager.locator('.connection-builder>summary').click();
 for(const part of ['1Lz','1Lo','1Lo'])await manager.getByRole('button',{name:`Next part: ${part}`,exact:true}).click();
 await expect(manager.getByRole('button',{name:'Exclude connection 1Lz+1Lo+1Lo',exact:true})).toBeVisible();
 await manager.getByRole('button',{name:'Restore connection 1Lz+1Lo',exact:true}).click();expect((await workspace(page)).excludedConnections).toHaveLength(1);expect(creates).toBe(2);
});

test('excluding a connection removes duplicate required occurrences and undo restores exact program without losing later edits',async({page})=>{
 test.setTimeout(90000);await start(page);
 await openPicker(page);await page.getByTestId('element-query').fill('1T');await page.getByTestId('add-1T').click();
 await openPicker(page);const builder=page.getByTestId('program-connection-builder');await builder.locator('summary').click();
 for(const part of ['1Lz','1Lo'])await builder.getByRole('button',{name:`Next part: ${part}`,exact:true}).click();
 await builder.getByRole('button',{name:'Add connection 1Lz+1Lo',exact:true}).click();
 await openPicker(page);await page.getByTestId('element-query').fill('1F');await page.getByTestId('add-1F').click();
 await openPicker(page);await builder.getByRole('button',{name:'Add connection 1Lz+1Lo',exact:true}).click();
 await page.getByTestId('program-entry').last().getByRole('checkbox').uncheck();
 const before=await workspace(page);
 await page.getByTestId('calculate').click();await expect(page.getByTestId('adopt-result').first()).toBeEnabled({timeout:45000});
 const action=await connection(page,'1Lz+1Lo');await action.locator('xpath=ancestor::li[@data-result-id]').getByRole('checkbox').check();
 await page.getByTestId('open-comparison').click();const comparison=page.getByTestId('program-comparison');await comparison.locator('summary').click();
 await comparison.getByRole('button',{name:'Exclude connection 1Lz+1Lo',exact:true}).first().click();
 await expect(page.getByTestId('program-entry')).toHaveCount(2);await expect(page.getByTestId('exclusion-feedback')).toContainText('2');
 await page.getByTestId('program-entry').first().getByRole('checkbox').uncheck();before.entries[0].retained=false;
 await openPicker(page);await page.getByTestId('element-query').fill('1S');await page.getByTestId('add-1S').click();
 const added=(await workspace(page)).entries.at(-1);
 await page.getByTestId('undo-exclusion').click();
 const after=await workspace(page);expect(after.entries).toEqual([...before.entries,added]);expect(after.excludedConnections).toEqual(before.excludedConnections);
});

for(const width of [1280,390,320])test(`readable complete cards, comparison and exclusions at ${width}px`,async({page})=>{
 test.setTimeout(90000);await page.setViewportSize({width,height:width===1280?800:844});await start(page);
 await page.getByTestId('discover-programs').click();await expect(page.locator('.planning-search')).toContainText('All optimal compositions collected.',{timeout:45000});
 await mkdir('../docs/evidence/167',{recursive:true});
 const first=cards(page).locator('.search-results>li').first();
 await first.scrollIntoViewIfNeeded();await page.screenshot({path:`../docs/evidence/167/cards-${width}.png`,fullPage:false});
 const sequence=await connection(page,'1Lz+1A+1A+SEQ');await sequence.scrollIntoViewIfNeeded();
 await expect(sequence).toBeInViewport();const box=(await sequence.boundingBox())!;expect(box.height).toBeGreaterThanOrEqual(44);expect(box.width).toBeGreaterThanOrEqual(44);
 if(width<1100)expect(box.y+box.height).toBeLessThan((await page.getByRole('navigation',{name:'Planning areas'}).boundingBox())!.y);
 await sequence.focus();await page.keyboard.press('Tab');await page.keyboard.press('Shift+Tab');await expect(sequence).toHaveCSS('outline-style','solid');await page.screenshot({path:`../docs/evidence/167/sequence-${width}.png`});
 await page.keyboard.press('Enter');await expect(sequence).toHaveCount(0);
 const excluded=cards(page).getByRole('button',{name:'Excluded 1Lz+1A+1A+SEQ',exact:true}).first();await expect(excluded).toBeFocused();
 await expect(excluded).toHaveAttribute('aria-disabled','true');
 await page.getByTestId('undo-exclusion').scrollIntoViewIfNeeded();await page.screenshot({path:`../docs/evidence/167/undo-${width}.png`});
 await page.getByTestId('undo-exclusion').click();await expect(page.getByTestId('exclusion-feedback')).toBeFocused();
 await (await connection(page,'1Lz+1A+1A+SEQ')).click();
 await (await connection(page,'1Lz+1Lo')).click();
 await page.getByTestId('undo-exclusion').click();expect((await workspace(page)).excludedConnections).toHaveLength(1);
 const checkbox=cards(page).getByRole('checkbox').first();await checkbox.check();
 await page.getByTestId('open-comparison').click();const comparison=page.getByTestId('program-comparison');await comparison.scrollIntoViewIfNeeded();await page.screenshot({path:`../docs/evidence/167/comparison-${width}.png`});
 await page.getByTestId('panel-exclusions').click();await expect(page.getByTestId('connection-exclusions')).toContainText('solo jumps and longer connections');
 await page.screenshot({path:`../docs/evidence/167/exclusions-${width}.png`});
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 for(const locale of ['de','ru','en']){await page.getByLabel('Language').selectOption(locale);await expect(page.getByTestId('exclusion-feedback')).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);}
 await page.getByLabel('Category').selectOption({label:'Adult Bronze'});await expect(page.getByTestId('connection-exclusions').locator('[data-restore]')).toHaveCount(1);
 await page.getByTestId('connection-exclusions').locator('[data-restore]').click();expect((await workspace(page)).excludedConnections).toHaveLength(0);
});

test('direct exclusions reject a delayed real collection response and dispose the obsolete session',async({page})=>{
 test.setTimeout(90000);await start(page);
 let release=()=>{},seen=()=>{},heldOnce=false,creates=0,owner:{id:string;accessToken:string}|undefined;
 const held=new Promise<void>(r=>release=r),captured=new Promise<void>(r=>seen=r);
 page.on('response',async r=>{if(new URL(r.url()).pathname==='/api/v1/searches'&&r.request().method()==='POST'){creates++;owner=await r.json();}});
 await page.route('**/api/v1/searches/*/updates?*',async route=>{
  const response=await route.fetch();
  if(!heldOnce&&Number(new URL(route.request().url()).searchParams.get('after'))>0){heldOnce=true;seen();await held;}
  await route.fulfill({response});
 });
 await page.getByTestId('calculate').click();await captured;
 const ids=await cards(page).locator('[data-result-id]').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-result-id')));
 await cards(page).getByTestId('exclude-result').first().click();
 await expect(page.getByTestId('search-stale')).toBeVisible();release();
 await expect.poll(async()=> (await page.request.get(`/api/v1/searches/${owner!.id}/updates`,{headers:{Authorization:`Bearer ${owner!.accessToken}`}})).status()).toBe(404);
 expect(await cards(page).locator('[data-result-id]').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-result-id')))).toEqual(ids);
 await expect(cards(page).getByTestId('adopt-result').first()).toBeDisabled();expect(creates).toBe(1);
});
