import {test,expect,type Page} from '@playwright/test';
import {openPicker,setEnabled,setMaximum} from './picker';
import {mkdir} from 'node:fs/promises';

const workspace=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!));
async function start(page:Page){await page.goto('/');await page.getByLabel('Language').selectOption('en');await page.getByLabel('Category').selectOption({label:'Adult Silver'});}
async function add(page:Page,code:string){await openPicker(page);await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();}
async function pair(page:Page,first='1Lz',second='1Lo'){
 await openPicker(page);const builder=page.getByTestId('program-connection-builder');
 if(!await builder.locator('details').evaluate((el:HTMLDetailsElement)=>el.open))await builder.locator('summary').click();
 await builder.getByRole('button',{name:'Start again',exact:true}).click();
 for(const part of [first,second])await builder.getByRole('button',{name:`Next part: ${part}`,exact:true}).click();
 await builder.getByRole('button',{name:`Add connection ${first}+${second}`,exact:true}).click();await page.keyboard.press('Escape');
}

test('disabling a component removes whole duplicate required compounds and undo preserves later edits',async({page})=>{
 await start(page);await add(page,'1T');await pair(page);await add(page,'1F');await pair(page);
 await page.getByTestId('program-entry').last().getByRole('checkbox').uncheck();
 await page.getByText('Program is fully described',{exact:true}).click();
 const before=await workspace(page);
 await setEnabled(page,'Lo',false);
 await expect(page.getByTestId('program-entry')).toHaveCount(2);
 await expect(page.getByTestId('ability-feedback')).toContainText('1Lz+1Lo');
 expect((await workspace(page)).entries).toEqual([before.entries[0],before.entries[2]]);
 await page.getByTestId('program-entry').first().getByRole('checkbox').uncheck();before.entries[0].retained=false;
 await add(page,'1S');const added=(await workspace(page)).entries.at(-1);
 await page.getByTestId('undo-ability').click();
 expect((await workspace(page)).entries).toEqual([...before.entries,added]);
 expect((await workspace(page)).personalLimits).toEqual(before.personalLimits);
 expect((await workspace(page)).complete).toBe(true);
 await expect(page.getByTestId('ability-feedback')).toBeFocused();
});

test('lowering jump and spin ceilings removes only the unavailable occurrences',async({page})=>{
 await start(page);await page.getByLabel('Category').selectOption({label:'Adult Masters'});
 await setMaximum(page,'T',2);await setMaximum(page,'FSSp',2);
 await add(page,'2T');await pair(page,'2T','1Lo');await add(page,'1T');await add(page,'FSSp2');await add(page,'FSSpB');
 const before=await workspace(page);
 await page.getByTestId('fader-T').press('ArrowLeft');await expect(page.getByTestId('program-entry')).toHaveCount(3);
 expect((await workspace(page)).entries).toEqual(before.entries.slice(2));
 await page.getByTestId('undo-ability').click();expect((await workspace(page)).entries).toEqual(before.entries);
 await page.getByTestId('fader-FSSp').press('ArrowLeft');await expect(page.getByTestId('program-entry')).toHaveCount(4);
 expect((await workspace(page)).entries).toEqual(before.entries.filter((e:{code:string})=>e.code!=='FSSp2'));
 await page.getByTestId('undo-ability').click();expect((await workspace(page)).entries).toEqual(before.entries);
});

test('late ability assessment cannot remove restored entries after undo',async({page})=>{
 await start(page);await pair(page);const before=await workspace(page);
 let release=()=>{},seen=()=>{};const held=new Promise<void>(r=>release=r),captured=new Promise<void>(r=>seen=r);let once=false;
 await page.route('**/api/v1/program-assessments',async route=>{
  const response=await route.fetch();
  if(!once&&route.request().postDataJSON().personalLimits.jumps.Lo?.enabled===false){once=true;seen();await held;}
  await route.fulfill({response});
 });
 await setEnabled(page,'Lo',false);await captured;await page.getByTestId('undo-ability').click();release();
 await expect(page.getByTestId('assessment')).not.toContainText('Assessing');
 expect((await workspace(page)).entries).toEqual(before.entries);expect((await workspace(page)).personalLimits).toEqual(before.personalLimits);
 await expect(page.getByTestId('ability-feedback')).toContainText('undone');
});

test('rapid changes remove each affected whole and undo only the latest limit',async({page})=>{
 await start(page);await add(page,'1Lo');await add(page,'SSp2');await add(page,'1F');await pair(page,'1Lo','1T');const before=await workspace(page);
 let release=()=>{},seen=()=>{};const held=new Promise<void>(r=>release=r),captured=new Promise<void>(r=>seen=r);let once=false;
 await page.route('**/api/v1/program-assessments',async route=>{
  const response=await route.fetch();if(!once&&route.request().postDataJSON().personalLimits.jumps.Lo?.enabled===false){once=true;seen();await held;}await route.fulfill({response});
 });
 await setEnabled(page,'Lo',false);await captured;await setEnabled(page,'SSp',false);await setEnabled(page,'T',false);
 await expect(page.getByTestId('program-entry')).toHaveCount(1);release();
 await page.getByTestId('undo-ability').click();await expect(page.getByTestId('program-entry')).toHaveCount(1);
 expect((await workspace(page)).entries).toEqual([before.entries[2]]);expect((await workspace(page)).personalLimits.jumps.Lo.enabled).toBe(false);
 expect((await workspace(page)).personalLimits.spins.SSp.enabled).toBe(false);
});

test('step and unlevelled settings retain whole-entry removal and completeness edits',async({page})=>{
 await start(page);await page.getByLabel('Category').selectOption({label:'Adult Masters'});await setMaximum(page,'StSq',2);
 await add(page,'StSq2');await add(page,'ChSp1');await add(page,'1T');const before=await workspace(page);
 await page.getByTestId('fader-StSq').press('ArrowLeft');await expect(page.getByTestId('program-entry')).toHaveCount(2);
 await page.getByText('Program is fully described',{exact:true}).click();await page.getByTestId('undo-ability').click();
 expect((await workspace(page)).entries).toEqual(before.entries);expect((await workspace(page)).complete).toBe(true);
 await setEnabled(page,'ChSp',false);await expect(page.getByTestId('program-entry')).toHaveCount(2);
 await page.getByTestId('undo-ability').click();expect((await workspace(page)).entries).toEqual(before.entries);
});

test('ability changes stale a delayed real search without starting another search',async({page})=>{
 test.setTimeout(90000);await start(page);
 // Keep the real search small; presentation and slider commits must never create one.
 for(const id of ['T','S','Lz'])await setEnabled(page,id,false);
 for(const id of ['F','Lo','A'])await setMaximum(page,id,1);
 await setEnabled(page,'W',false);
 let creates=0,release=()=>{},seen=()=>{},once=false,owner:{id:string;accessToken:string}|undefined;
 const held=new Promise<void>(r=>release=r),captured=new Promise<void>(r=>seen=r);
 page.on('response',async r=>{if(new URL(r.url()).pathname==='/api/v1/searches'&&r.request().method()==='POST'){creates++;owner=await r.json();}});
 await page.route('**/api/v1/searches/*/updates?*',async route=>{
  const response=await route.fetch();if(!once&&Number(new URL(route.request().url()).searchParams.get('after'))>0){once=true;seen();await held;}await route.fulfill({response});
 });
 await page.getByTestId('calculate').click();await captured;
 try{
  await setEnabled(page,'Lo',false);await expect(page.getByTestId('search-stale')).toBeVisible();release();
  await expect(page.getByTestId('adopt-result').first()).toBeDisabled();
  await expect.poll(async()=> (await page.request.get(`/api/v1/searches/${owner!.id}/updates`,{headers:{Authorization:`Bearer ${owner!.accessToken}`}})).status()).toBe(404);
  await page.getByLabel('Language').selectOption('de');await page.getByTestId('panel-ability').click();expect(creates).toBe(1);
 }finally{release();if(owner)await page.request.delete(`/api/v1/searches/${owner.id}`,{headers:{Authorization:`Bearer ${owner.accessToken}`}});}
});

for(const width of [1280,390,320])test(`ability labels, values and focus remain readable at ${width}px`,async({page})=>{
 await page.setViewportSize({width,height:width===1280?800:844});await start(page);
 await mkdir('../docs/evidence/168',{recursive:true});
 for(const locale of ['en','de','ru']){
  await page.getByLabel('Language').selectOption(locale);
  await page.getByTestId('fader-T').scrollIntoViewIfNeeded();await page.screenshot({path:`../docs/evidence/168/jumps-${width}-${locale}.png`});
  const slider=page.getByTestId('fader-FCCoSp');await slider.scrollIntoViewIfNeeded();await slider.focus();await page.keyboard.press('Tab');await page.keyboard.press('Shift+Tab');
  await page.screenshot({path:`../docs/evidence/168/ability-${width}-${locale}.png`});
  const label=page.getByTestId('form-FCCoSp').locator('.form-name');
  expect(Number.parseFloat(await label.evaluate(el=>getComputedStyle(el).fontSize))).toBeGreaterThanOrEqual(16);
  await expect(slider).toHaveCSS('outline-style','solid');
  const box=(await slider.boundingBox())!;expect(box.width).toBeGreaterThanOrEqual(44);expect(box.height).toBeGreaterThanOrEqual(44);
  if(width<1100)expect(box.y+box.height).toBeLessThan((await page.getByRole('navigation').boundingBox())!.y);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }
 await page.getByLabel('Language').selectOption('en');await page.getByTestId('panel-program').click();await pair(page);
 await page.getByTestId('panel-ability').click();await setEnabled(page,'Lo',false);
 await expect(page.getByTestId('ability-feedback')).toContainText('1Lz+1Lo');await page.getByTestId('undo-ability').scrollIntoViewIfNeeded();
 await page.screenshot({path:`../docs/evidence/168/removal-${width}.png`});await page.getByTestId('undo-ability').click();
 await page.getByTestId('panel-program').click();await expect(page.getByTestId('program-entry')).toHaveCount(1);
});

test.describe('touch interaction',()=>{
 test.use({hasTouch:true,viewport:{width:390,height:844}});
 test('tapping a range changes the committed ability without extra controls',async({page})=>{
  await start(page);const slider=page.getByTestId('fader-SSp');await slider.scrollIntoViewIfNeeded();
  await slider.tap({position:{x:8,y:22}});await expect(slider).toHaveAttribute('aria-valuetext','Off');
  const box=(await slider.boundingBox())!;await slider.tap({position:{x:box.width-8,y:22}});await expect(slider).toHaveAttribute('aria-valuetext','Maximum: 2');
  await slider.press('Home');await slider.press('ArrowRight');await expect(slider).toHaveAttribute('aria-valuetext','Maximum: Basic');
 });
});
