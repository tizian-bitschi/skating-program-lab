import {test,expect} from '@playwright/test';
import {openPicker,configureSilver} from './picker';

test('phone panel changes preserve an in-flight real search and editor query',async({page})=>{
 await page.setViewportSize({width:390,height:844});await page.goto('/');await page.getByLabel('Language').selectOption('en');
 let release=()=>{},seen=()=>{},creates=0,deletes=0;
 const held=new Promise<void>(resolve=>release=resolve),captured=new Promise<void>(resolve=>seen=resolve);
 await page.route('**/api/v1/searches',async route=>{creates++;const response=await route.fetch();seen();await held;await route.fulfill({response});});
 page.on('request',request=>{if(request.method()==='DELETE'&&request.url().includes('/api/v1/searches/'))deletes++;});
 await page.getByTestId('discover-programs').click();await captured;
 for(const panel of ['program','ability','exclusions','suggestions'])await page.getByTestId(`panel-${panel}`).click();
 await expect(page.getByTestId('calculate')).toBeDisabled();expect(creates).toBe(1);expect(deletes).toBe(0);
 release();await expect(page.getByTestId('adopt-result').first()).toBeEnabled({timeout:30000});
 await page.getByTestId('panel-program').click();await openPicker(page);await page.getByTestId('element-query').fill('Lutz');await page.keyboard.press('Escape');
 await page.getByTestId('panel-ability').click();await page.getByTestId('panel-program').click();await openPicker(page);await expect(page.getByTestId('element-query')).toHaveValue('Lutz');
 expect(creates).toBe(1);expect(deletes).toBe(0);
});

for(const width of [1280,390,320]){
 test(`both planning paths preserve required occurrences and search ownership at ${width}px`,async({page},info)=>{
  test.setTimeout(90000);
  await page.setViewportSize({width,height:width===1280?800:844});
  await page.goto('/');await page.getByLabel('Language').selectOption('en');
  await configureSilver(page);
  let creates=0,deletes=0;const bodies:Record<string,unknown>[]=[];
  page.on('request',request=>{const path=new URL(request.url()).pathname;if(path==='/api/v1/searches'&&request.method()==='POST'){creates++;bodies.push(request.postDataJSON());}if(path.startsWith('/api/v1/searches/')&&request.method()==='DELETE')deletes++;});
  await expect(page.getByTestId('discover-programs')).toBeEnabled();
  await page.screenshot({path:info.outputPath('ability.png'),fullPage:true});
  await page.getByTestId('panel-ability').click();await page.evaluate(()=>scrollTo(0,0));await page.screenshot({path:info.outputPath('ability-viewport.png')});
  await page.getByTestId('discover-programs').click();
  await expect(page.getByTestId('adopt-result').first()).toBeEnabled({timeout:30000});
  expect(bodies[0]!.requiredElements).toEqual([]);
  const resultBrowser=page.locator('.planning-search>.result-browser');
  await resultBrowser.getByRole('checkbox').first().check();
  await resultBrowser.getByRole('button',{name:'Next programs'}).click();
  await resultBrowser.locator('[data-result-id]').first().scrollIntoViewIfNeeded();
  const readingY=await page.evaluate(()=>scrollY),beforeDeletes=deletes;
  await page.getByTestId('panel-ability').click();
  await page.getByTestId('panel-exclusions').click();
  await page.getByTestId('panel-program').click();
  await page.getByTestId('panel-suggestions').click();
  expect(creates).toBe(1);expect(deletes).toBe(beforeDeletes);
  await expect(page.getByTestId('program-comparison')).toBeVisible();
  await expect(resultBrowser.getByRole('spinbutton',{name:'Page'})).toHaveValue('2');
  if(width<1100)expect(Math.abs(await page.evaluate(()=>scrollY)-readingY)).toBeLessThan(3);
  await page.screenshot({path:info.outputPath('suggestions.png'),fullPage:true});
  await page.screenshot({path:info.outputPath('suggestions-viewport.png')});
  await page.getByTestId('panel-ability').click();await page.getByTestId('specify-elements').click();
  for(let i=0;i<2;i++){await openPicker(page);await page.getByTestId('element-query').fill('1Lo');await page.getByTestId('add-1Lo').click();}
  const occurrences=page.getByTestId('program-entry');
  await expect(occurrences).toHaveCount(2);await expect(occurrences.nth(0).getByRole('checkbox')).toBeChecked();await expect(occurrences.nth(1).getByRole('checkbox')).toBeChecked();
  const ids=await occurrences.evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-entry-id')));
  expect(new Set(ids).size).toBe(2);
  await occurrences.first().getByRole('checkbox').uncheck();await occurrences.first().getByRole('checkbox').check();
  await expect(page.getByTestId('assessment')).toContainText('Credited BV');
  await page.screenshot({path:info.outputPath('required.png'),fullPage:true});
  await page.getByTestId('complete-program').scrollIntoViewIfNeeded();await page.screenshot({path:info.outputPath('required-viewport.png')});
  await page.getByTestId('complete-program').click();
  await expect(page.getByTestId('adopt-result').first()).toBeEnabled({timeout:30000});
  expect(bodies[1]!.requiredElements).toEqual(ids.map(id=>({id,elementKey:expect.stringContaining(':1Lo:')})));
  await page.getByTestId('adopt-result').first().click();
  for(const id of ids)await expect(page.locator(`[data-entry-id="${id}"]`).getByRole('checkbox')).toBeChecked();
  await expect(page.getByTestId('assessment')).toContainText('Construction complies');
  expect(creates).toBe(2);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(width<1100){
   const nav=await page.getByRole('navigation',{name:'Planning areas'}).boundingBox();
   for(const name of ['ability','program','suggestions','exclusions']){const button=page.getByTestId(`panel-${name}`);const box=(await button.boundingBox())!;expect(box.height).toBeGreaterThanOrEqual(44);expect(box.width).toBeGreaterThanOrEqual(44);}
   await page.getByTestId('complete-program').focus();
   const box=(await page.getByTestId('complete-program').boundingBox())!;expect(box.y+box.height).toBeLessThan(nav!.y);
   await page.getByTestId('panel-exclusions').focus();await page.keyboard.press('Enter');await expect(page.getByTestId('panel-exclusions')).toHaveAttribute('aria-current','page');
  }
 });
}
