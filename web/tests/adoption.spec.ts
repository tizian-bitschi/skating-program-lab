import {test,expect,type Page} from '@playwright/test';
import {openPicker,setEnabled,configureSilver} from './picker';

async function silver(page:Page){
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 await configureSilver(page);
}

test('releasing and adding requirements invalidates adoption and submits the new occurrences',async({page})=>{
 test.setTimeout(90000);await silver(page);await page.getByTestId('specify-elements').click();
 await add(page,'1Lo');await add(page,'1Lo');
 const original=await program(page);let creates=0;
 page.on('request',request=>{if(new URL(request.url()).pathname==='/api/v1/searches'&&request.method()==='POST')creates++;});
 await page.getByTestId('complete-program').click();await expect(page.getByTestId('adopt-result').first()).toBeEnabled({timeout:30000});
 await page.getByTestId('adopt-result').first().click();
 await page.locator(`[data-entry-id="${original[0]!.id}"]`).getByRole('checkbox').uncheck();
 await expect(page.getByTestId('undo-adoption')).toHaveCount(0);
 await expect(page.getByTestId('adoption-feedback')).toContainText('no longer available');
 await add(page,'1T');const changed=await program(page),required=changed.filter(e=>e.required);
 await page.getByTestId('panel-suggestions').click();await expect(page.getByTestId('search-stale')).toBeVisible();
 for(const button of await page.getByTestId('adopt-result').all())await expect(button).toBeDisabled();
 expect(creates).toBe(1);
 const submitted=page.waitForRequest(request=>new URL(request.url()).pathname==='/api/v1/searches'&&request.method()==='POST');
 await page.getByTestId('calculate').click();const input=(await submitted).postDataJSON();
 expect(input.requiredElements).toEqual(required.map(e=>({id:e.id,elementKey:expect.stringContaining(`:${e.code}:`)})));
 await expect(page.getByTestId('adopt-result').first()).toBeEnabled({timeout:30000});await page.getByTestId('adopt-result').first().click();
 const adopted=await program(page);for(const occurrence of required)expect(adopted).toContainEqual(occurrence);
 expect(adopted.some(e=>e.id===original[0]!.id)).toBe(false);
 await page.getByTestId('undo-adoption').click();expect(await program(page)).toEqual(changed);expect(creates).toBe(2);
});

for(const edit of ['remove','reorder','complete','ability','category','exclusion','rotation'] as const){
 test(`adoption undo cannot overwrite a later ${edit} edit`,async({page})=>{
  test.setTimeout(60000);await silver(page);await page.getByTestId('discover-programs').click();
  await expect(page.getByTestId('adopt-result').first()).toBeEnabled({timeout:30000});await page.getByTestId('adopt-result').first().click();
  await expect(page.getByTestId('undo-adoption')).toBeVisible();
  const first=page.getByTestId('program-entry').first(),id=await first.getAttribute('data-entry-id');
  if(edit==='remove')await first.getByRole('button',{name:'Remove',exact:true}).click();
  if(edit==='reorder')await first.getByRole('button',{name:'Move down'}).click();
  if(edit==='complete')await page.getByLabel('Program is fully described',{exact:true}).uncheck();
  if(edit==='ability')await setEnabled(page,'A',false);
  if(edit==='category')await page.getByLabel('Category').selectOption({label:'Adult Bronze'});
  if(edit==='rotation'){await page.getByText('More options',{exact:true}).click();await page.getByRole('switch').check();}
  if(edit==='exclusion'){
   const manager=page.getByTestId('connection-exclusions');await manager.getByText('Excluded connections',{exact:true}).last().click();
   await manager.getByRole('button',{name:'Next part: 1Lz',exact:true}).click();await manager.getByRole('button',{name:'Next part: 1Lo',exact:true}).click();
   await manager.getByRole('button',{name:'Exclude connection 1Lz+1Lo',exact:true}).click();
  }
  await expect(page.getByTestId('undo-adoption')).toHaveCount(0);await expect(page.getByTestId('adoption-feedback')).toContainText('no longer available');
  if(edit==='remove')await expect(page.locator(`[data-entry-id="${id}"]`)).toHaveCount(0);
  if(edit==='reorder')expect((await program(page))[1]!.id).toBe(id);
  if(edit==='complete')await expect(page.getByLabel('Program is fully described',{exact:true})).not.toBeChecked();
 });
}

test.describe('localized phone touch and keyboard adoption',()=>{
 test.use({hasTouch:true,viewport:{width:390,height:844}});
 for(const [locale,adopt,undo,restored,assessed] of [
  ['en','Use this program','Undo adoption','Previous program restored.','Construction complies'],
  ['de','Dieses Programm übernehmen','Übernahme rückgängig machen','Vorheriges Programm wiederhergestellt.','Aufbau entspricht'],
  ['ru','Использовать эту программу','Отменить принятие','Предыдущая программа восстановлена.','Построение соответствует']
 ]){
  test(`${locale} labels, touch targets and keyboard focus`,async({page},info)=>{
   test.setTimeout(60000);await silver(page);await page.getByLabel('Language').selectOption(locale!);
   await page.getByTestId('discover-programs').tap();const action=page.getByTestId('adopt-result').first();
   await expect(action).toBeEnabled({timeout:30000});await expect(action).toHaveAccessibleName(adopt!);
   const box=(await action.boundingBox())!;expect(box.height).toBeGreaterThanOrEqual(44);expect(box.width).toBeGreaterThanOrEqual(44);
   await action.tap();const restore=page.getByRole('button',{name:undo!,exact:true});await expect(restore).toBeVisible();
   await expect(page.getByTestId('assessment')).toContainText(assessed!);
   // Tab from the focused panel heading reaches the undo action.
   await page.keyboard.press('Tab');await expect(restore).toBeFocused();await expect(restore).toHaveCSS('outline-style','solid');
   await page.screenshot({path:info.outputPath(`undo-${locale}.png`)});
   await page.keyboard.press('Enter');await expect(page.getByTestId('program-entry')).toHaveCount(0);
   await expect(page.getByTestId('adoption-feedback')).toHaveText(restored!);await expect(page.getByTestId('open-element-picker')).toBeFocused();
  });
 }
});
async function add(page:Page,code:string){await openPicker(page);await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();}
async function program(page:Page){
 return page.getByTestId('program-entry').evaluateAll(nodes=>nodes.map(node=>({id:node.getAttribute('data-entry-id'),code:node.querySelector('code')!.textContent,required:node.querySelector<HTMLInputElement>('input')!.checked})));
}

for(const width of [1280,390,320]){
 test(`adopt A then B and undo preserves duplicate required occurrences at ${width}px`,async({page},info)=>{
  test.setTimeout(90000);await page.setViewportSize({width,height:width===1280?800:844});await silver(page);
  await page.getByTestId('specify-elements').click();
  await add(page,'1Lo');await add(page,'1Lo');await add(page,'1T');
  await page.getByTestId('program-entry').last().getByRole('checkbox').uncheck();
  const original=await program(page),required=original.slice(0,2);
  await expect(page.getByTestId('assessment')).toContainText('Draft');
  await page.screenshot({path:info.outputPath('original.png')});
  await page.getByTestId('complete-program').click();
  await expect(page.getByTestId('adopt-result').nth(1)).toBeEnabled({timeout:30000});
  await page.getByTestId('adopt-result').first().focus();await page.keyboard.press('Enter');
  const a=await program(page);for(const occurrence of required)expect(a).toContainEqual(occurrence);
  expect(a.filter(e=>e.required)).toEqual(required);expect(a).not.toContainEqual(original[2]);
  await expect(page.getByTestId('assessment')).toContainText('Construction complies');
  await page.getByTestId('undo-adoption').click();
  expect(await program(page)).toEqual(original);await expect(page.getByTestId('assessment')).toContainText('Draft');
  await page.getByTestId('panel-suggestions').click();await page.getByTestId('adopt-result').first().click();
  const restoredA=await program(page);
  await page.getByTestId('panel-suggestions').click();
  await expect(page.getByTestId('adopt-result').nth(1)).toHaveAccessibleDescription(/Replaces interchangeable entries/);
  await page.getByTestId('adopt-result').nth(1).scrollIntoViewIfNeeded();await page.screenshot({path:info.outputPath('suggestions.png')});
  await page.getByTestId('adopt-result').nth(1).click();
  const b=await program(page);for(const occurrence of required)expect(b).toContainEqual(occurrence);
  await expect(page.getByTestId('assessment')).toContainText('Construction complies');
  expect(b.filter(e=>e.required)).toEqual(required);expect(b.length).toBe(restoredA.length);
  expect(b.map(e=>e.code)).not.toEqual(restoredA.map(e=>e.code));
  const undo=page.getByTestId('undo-adoption');await expect(undo).toBeVisible();await undo.focus();
  await expect(undo).toBeFocused();const box=(await undo.boundingBox())!;expect(box.height).toBeGreaterThanOrEqual(44);expect(box.width).toBeGreaterThanOrEqual(44);
  if(width<1100)expect(box.y+box.height).toBeLessThan((await page.getByRole('navigation').boundingBox())!.y);
  await page.screenshot({path:info.outputPath('adopted.png')});
  await page.keyboard.press('Enter');expect(await program(page)).toEqual(restoredA);
  await expect(page.getByTestId('assessment')).toContainText('Construction complies');
  await expect(page.getByTestId('adoption-feedback')).toContainText('Previous program restored');
  await page.screenshot({path:info.outputPath('restored.png')});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 });
}
