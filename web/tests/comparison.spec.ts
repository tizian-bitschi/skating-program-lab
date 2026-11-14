import {test,expect} from './search-fixture';
import {configureSilver} from './picker';
import {mkdir} from 'node:fs/promises';

test('phone comparison returns to the selected result page and reading position',async({page})=>{
 test.setTimeout(90000);
 await page.setViewportSize({width:390,height:844});await page.goto('/');await page.getByLabel('Language').selectOption('en');await configureSilver(page);
 await page.getByTestId('discover-programs').click();await expect(page.locator('.planning-search')).toContainText('All optimal compositions collected.',{timeout:45000});
 const cards=page.locator('.planning-search>.result-browser');
 await cards.getByRole('checkbox').first().check();await cards.getByRole('button',{name:'Next programs',exact:true}).click();await cards.getByRole('checkbox').first().check();
 const open=page.getByTestId('open-comparison');await open.focus();await open.scrollIntoViewIfNeeded();const y=await page.evaluate(()=>scrollY);
 await page.keyboard.press('Enter');const comparison=page.getByTestId('program-comparison');await expect(comparison).toBeVisible();await expect(cards).toBeHidden();
 await expect(comparison.getByRole('heading',{name:'Differences',exact:true})).toHaveCount(2);
 await comparison.getByText('Identical content',{exact:false}).first().click();
 await page.getByTestId('panel-ability').click();await page.getByTestId('panel-suggestions').click();await expect(comparison).toBeVisible();
 await page.getByTestId('close-comparison').click();await expect(open).toBeFocused();await expect(cards.getByRole('spinbutton')).toHaveValue('2');await expect(cards.getByRole('checkbox').first()).toBeChecked();expect(Math.abs(await page.evaluate(()=>scrollY)-y)).toBeLessThan(3);
});

for(const width of [1280,390,320])test(`real comparison differences, exclusions and localized focus at ${width}px`,async({page})=>{
 test.setTimeout(120000);await page.setViewportSize({width,height:width===1280?800:844});
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await configureSilver(page);
 let creates=0;page.on('request',r=>{if(new URL(r.url()).pathname==='/api/v1/searches'&&r.method()==='POST')creates++;});
 await page.getByTestId('discover-programs').click();await expect(page.locator('.planning-search')).toContainText('All optimal compositions collected.',{timeout:45000});
 const cards=page.locator('.planning-search>.result-browser');
 for(const code of ['1Lz+1A+1A+SEQ','1Lz+1Lo']){
  for(let i=0;i<10;i++){
   const action=cards.getByRole('button',{name:`Exclude connection ${code}`,exact:true}).first();
   if(await action.count()){await action.locator('xpath=ancestor::li[@data-result-id]').getByRole('checkbox').check();break;}
   await cards.getByRole('button',{name:'Next programs',exact:true}).click();
  }
 }
 if(await cards.getByRole('checkbox',{checked:true}).count()===1)await cards.getByRole('checkbox',{checked:false}).first().check();
 await expect(cards.getByRole('checkbox',{checked:true})).not.toHaveCount(0);
 await page.getByTestId('open-comparison').click();const comparison=page.getByTestId('program-comparison');
 await expect(comparison.locator('[data-result-id]')).toHaveCount(2);
 for(const details of await comparison.locator('details').all())await details.locator('summary').click();
 await expect(comparison.getByText('Direct step into Axel assumed',{exact:true}).first()).toBeVisible();
 await mkdir('../docs/evidence/169',{recursive:true});
 for(const locale of ['en','de','ru']){
  await page.getByLabel('Language').selectOption(locale);const back=page.getByTestId('close-comparison');await back.focus();await page.keyboard.press('Tab');await page.keyboard.press('Shift+Tab');await back.evaluate(el=>el.scrollIntoView({block:'center'}));
  expect(await back.evaluate(el=>{const r=el.getBoundingClientRect();return document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)===el;})).toBe(true);await expect(back).toBeFocused();await expect(back).toHaveCSS('outline-style','solid');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(width<1100)expect((await back.boundingBox())!.y+(await back.boundingBox())!.height).toBeLessThan((await page.getByTestId('panel-suggestions').boundingBox())!.y);
  await page.screenshot({path:`../docs/evidence/169/comparison-${width}-${locale}.png`});
  const connection=comparison.getByTestId('exclude-result').first();await connection.focus();await connection.evaluate(el=>el.scrollIntoView({block:'center'}));
  expect(await connection.evaluate(el=>{const r=el.getBoundingClientRect();return document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)===el;})).toBe(true);
  const target=(await connection.boundingBox())!;expect(target.width).toBeGreaterThanOrEqual(44);expect(target.height).toBeGreaterThanOrEqual(44);
  if(width<1100)expect(target.y+target.height).toBeLessThan((await page.getByTestId('panel-suggestions').boundingBox())!.y);
  await page.screenshot({path:`../docs/evidence/169/connection-${width}-${locale}.png`});
 }
 await page.getByLabel('Language').selectOption('en');
 for(const code of ['1Lz+1A+1A+SEQ','1Lz+1Lo']){
  const action=comparison.getByRole('button',{name:`Exclude connection ${code}`,exact:true}).first();await action.focus();await page.keyboard.press('Enter');
  await expect(comparison.getByRole('button',{name:`Excluded ${code}`,exact:true}).first()).toHaveAttribute('aria-disabled','true');
 }
 expect(creates).toBe(1);await expect(comparison.getByTestId('adopt-comparison').first()).toBeDisabled();await expect(page.getByTestId('search-stale')).toBeVisible();
 await page.getByTestId('undo-exclusion').click();await expect(comparison.getByRole('button',{name:'Exclude connection 1Lz+1Lo',exact:true}).first()).toBeEnabled();
 await page.getByTestId('close-comparison').click();await page.getByRole('button',{name:'Update suggestions',exact:true}).click();await expect(page.locator('.planning-search')).toContainText('All optimal compositions collected.',{timeout:45000});expect(creates).toBe(2);
});
