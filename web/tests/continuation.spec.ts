import {test,expect} from './search-fixture';
import type {Page} from '@playwright/test';
import {configureSilver,openPicker,setEnabled} from './picker';
import {mkdir} from 'node:fs/promises';

const program=(page:Page)=>page.getByTestId('program-entry').evaluateAll(nodes=>nodes.map(node=>({id:node.getAttribute('data-entry-id'),code:node.querySelector('code')!.textContent,required:node.querySelector<HTMLInputElement>('input')!.checked})));
const cards=(page:Page)=>page.locator('.planning-search>.result-browser');
async function add(page:Page,code:string){await openPicker(page);await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();}
async function findConnection(page:Page,code:string){
 const list=cards(page);
 const number=list.getByRole('spinbutton');if(await number.count()){await number.fill('1');await number.press('Tab');}
 while(true){
  const action=list.getByRole('button',{name:`Exclude connection ${code}`,exact:true}).first();if(await action.count())return action;
  const next=list.getByRole('button',{name:'Next programs',exact:true});await expect(next).toBeEnabled();await next.click();
 }
}
// A second page reads the saved workspace while the original page keeps its scoped undo.
async function verifySaved(page:Page){
 const expected=await program(page),saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!));
 expect(saved.entries.map((e:{id:string;code:string;retained:boolean})=>({id:e.id,code:e.code,required:e.retained}))).toEqual(expected);
 expect(Object.keys(saved).sort()).toEqual(['allowMixedDirections','complete','context','entries','excludedConnections','locale','personalLimits','version']);
 const restored=await page.context().newPage();let searches=0;
 restored.on('request',request=>{if(new URL(request.url()).pathname.startsWith('/api/v1/searches'))searches++;});
 try{
  await restored.goto(page.url());await expect(restored.getByTestId('workspace-storage')).toContainText('saved');
  expect(await program(restored)).toEqual(expected);
  expect(await restored.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!))).toEqual(saved);
  await expect(restored.getByLabel('Program is fully described',{exact:true})).toBeChecked({checked:saved.complete});
  expect(searches).toBe(0);
 }finally{await restored.close();}
}

for(const width of [1280,390,320])for(const path of ['discovery','required'] as const)test(`${path} continuation journey at ${width}px`,async({page})=>{
 test.setTimeout(180000);await page.setViewportSize({width,height:width===1280?800:844});
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await configureSilver(page);
 await expect(page.getByTestId('fader-ChSq')).not.toHaveValue('0');
 await mkdir('../docs/evidence/170',{recursive:true});
 await page.getByTestId('fader-Lz').focus();await page.screenshot({path:`../docs/evidence/170/${path}-ability-${width}.png`});
 let creates=0;const inputs:{requiredElements:{id:string}[];excludedConnections:string[]}[]=[];
 page.on('request',request=>{if(new URL(request.url()).pathname==='/api/v1/searches'&&request.method()==='POST'){creates++;inputs.push(request.postDataJSON());}});
 if(path==='required'){await page.getByTestId('specify-elements').click();await add(page,'ChSq1');}
 const required=await program(page);
 await page.getByTestId(path==='required'?'complete-program':'discover-programs').click();
 await expect(page.locator('.planning-search')).toContainText('All optimal compositions collected.',{timeout:45000});
 console.log(`${path}/${width}: ${await page.getByTestId('search-maximum').textContent()}; ${await page.locator('.planning-search [role=status]').first().textContent()}`);
 expect(inputs[0]!.requiredElements.map(e=>e.id)).toEqual(required.map(e=>e.id));
 for(const code of ['1Lz+1A+1A+SEQ','1Lz+1Lo'])await (await findConnection(page,code)).locator('xpath=ancestor::li[@data-result-id]').getByRole('checkbox').check();
 if(await cards(page).getByRole('checkbox',{checked:true}).count()===1)await cards(page).getByRole('checkbox',{checked:false}).first().check();
 const card=cards(page).locator('[data-result-id]').first();
 await card.locator('h3').evaluate(el=>el.scrollIntoView({block:'center'}));await page.screenshot({path:`../docs/evidence/170/${path}-card-${width}.png`});
 await card.getByTestId('adopt-result').focus();await card.getByTestId('adopt-result').evaluate(el=>el.scrollIntoView({block:'center'}));await page.screenshot({path:`../docs/evidence/170/${path}-card-end-${width}.png`});
 await page.getByTestId('open-comparison').click();const comparison=page.getByTestId('program-comparison');
 for(const summary of await comparison.locator('summary').all())await summary.click();
 await page.getByTestId('close-comparison').focus();await page.screenshot({path:`../docs/evidence/170/${path}-comparison-${width}.png`});
 await page.getByTestId('panel-program').click();
 await openPicker(page);const builder=page.getByTestId('program-connection-builder');await builder.locator('summary').click();
 for(const part of ['1Lz','1Lo'])await builder.getByRole('button',{name:`Next part: ${part}`,exact:true}).click();
 await builder.getByRole('button',{name:'Add connection 1Lz+1Lo',exact:true}).click();
 const before=await program(page);await page.getByTestId('panel-suggestions').click();await expect(comparison).toBeVisible();
 await comparison.getByRole('button',{name:'Exclude connection 1Lz+1Lo',exact:true}).first().click();
 expect(await program(page)).toEqual(before.filter(e=>e.code!=='1Lz+1Lo'));
 await verifySaved(page);await page.getByTestId('undo-exclusion').click();expect(await program(page)).toEqual(before);await verifySaved(page);
 for(const code of ['1Lz+1Lo','1Lz+1A+1A+SEQ'])await comparison.getByRole('button',{name:`Exclude connection ${code}`,exact:true}).first().click();
 expect(creates).toBe(1);await expect(page.getByTestId('search-stale')).toBeVisible();await expect(comparison.getByTestId('adopt-comparison').first()).toBeDisabled();
 await page.getByTestId('panel-exclusions').click();await page.screenshot({path:`../docs/evidence/170/${path}-exclusions-${width}.png`});
 await page.getByTestId('panel-suggestions').click();await page.getByTestId('close-comparison').click();
 await page.getByTestId('calculate').click();await expect(page.locator('.planning-search')).toContainText('All optimal compositions collected.',{timeout:45000});
 expect(creates).toBe(2);expect(inputs[1]!.excludedConnections).toHaveLength(2);
 console.log(`${path}/${width} after exclusions: ${await page.locator('.planning-search [role=status]').first().textContent()}`);
 do{
  for(const code of ['1Lz+1Lo','1Lz+1A+1A+SEQ'])await expect(cards(page).getByRole('button',{name:`Exclude connection ${code}`,exact:true})).toHaveCount(0);
  const next=cards(page).getByRole('button',{name:'Next programs',exact:true});if(!await next.count()||await next.isDisabled())break;await next.click();
 }while(true);
 await page.getByTestId('adopt-result').first().click();const adopted=await program(page);
 for(const entry of required)expect(adopted).toContainEqual(entry);await verifySaved(page);
 await page.getByTestId('undo-adoption').click();expect(await program(page)).toEqual(required);await verifySaved(page);
 await page.getByTestId('panel-suggestions').click();await page.getByTestId('adopt-result').first().click();
 const beforeReduction=await program(page);await page.getByTestId('panel-ability').click();await setEnabled(page,'Lo',false);
 await expect.poll(async()=> (await program(page)).filter(e=>e.code!.includes('Lo')).length).toBe(0);await verifySaved(page);
 await page.getByTestId('undo-ability').focus();await page.screenshot({path:`../docs/evidence/170/${path}-undo-${width}.png`});
 await page.keyboard.press('Enter');expect(await program(page)).toEqual(beforeReduction);await verifySaved(page);
 const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!));
 await page.reload();await page.getByTestId('panel-program').click();await expect(page.getByTestId('workspace-storage')).toContainText('saved');
 await expect(page.getByTestId('assessment')).toContainText('Construction complies');
 expect(await program(page)).toEqual(beforeReduction);expect(creates).toBe(2);
 expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!))).toEqual(saved);
 await page.getByTestId('panel-suggestions').click();await expect(page.getByTestId('adopt-result')).toHaveCount(0);await expect(page.getByTestId('open-comparison')).toHaveCount(0);
 for(const locale of ['en','de','ru']){
  await page.getByLabel('Language').selectOption(locale);const calculate=page.getByTestId('calculate');await calculate.focus();await page.keyboard.press('Tab');await page.keyboard.press('Shift+Tab');await calculate.scrollIntoViewIfNeeded();
  await expect(page.getByTestId('workspace-result-boundary')).toBeVisible();await expect(calculate).toHaveCSS('outline-style','solid');
  const box=(await calculate.boundingBox())!;expect(box.height).toBeGreaterThanOrEqual(44);
  if(width<1100)expect(box.y+box.height).toBeLessThan((await page.getByTestId('panel-suggestions').boundingBox())!.y);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:`../docs/evidence/170/${path}-reload-${width}-${locale}.png`});
 }
 expect(creates).toBe(2);
});

test('restored workspace explains the result boundary without starting calculation',async({page})=>{
 let searches=0;
 page.on('request',request=>{if(new URL(request.url()).pathname.startsWith('/api/v1/searches'))searches++;});
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 await expect(page.getByTestId('workspace-storage')).toContainText('saved');
 await page.reload();await page.getByTestId('panel-suggestions').click();
 await expect(page.getByTestId('workspace-result-boundary')).toContainText('After reload, calculate suggestions explicitly');
 await expect(page.getByTestId('calculate')).toBeEnabled();
 expect(searches).toBe(0);
});
