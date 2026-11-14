import {test,expect} from '@playwright/test';

// Presentation-only HTTP fixture: permutations of a composition are deliberately
// included here, although the real optimizer deduplicates them before delivery.
test('whole-element multiplicity, compound order and uncapped selection',async({page})=>{
 const solo={elementKey:'solo',code:'1T',requiredId:null},pair={elementKey:'pair',code:'1Lz+1Lo',requiredId:null},reverse={elementKey:'reverse',code:'1Lo+1Lz',requiredId:null};
 let requestId='',revision=0;
 await page.route('**/api/v1/searches/identity',r=>r.fulfill({json:{creationNamespace:'presentation'}}));
 await page.route('**/api/v1/searches',r=>{({requestId,revision}=r.request().postDataJSON());return r.fulfill({status:202,json:{id:'presentation',accessToken:'fixture',requestId,revision}});});
 await page.route('**/api/v1/searches/presentation/updates?*',r=>r.fulfill({json:{id:'presentation',requestId,revision,status:'completed',phase:'terminal',maximumBaseHundredths:310,confirmedCount:24,cursor:24,results:Array.from({length:24},(_,i)=>{
  const entries=i===22?[solo,pair,pair]:i===23?[solo,solo,reverse]:i%2?[pair,solo,solo]:[solo,solo,pair];
  return {id:`fixture-${i}`,entries,assessment:{totalHundredths:310,elements:entries.map(()=>({creditedHundredths:100,parts:[]}))}};
 })}}));
 await page.route('**/api/v1/searches/presentation',r=>r.fulfill({json:{}}));
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await page.getByTestId('calculate').click();
 const cards=page.locator('.planning-search>.result-browser');
 await expect(cards.getByRole('checkbox')).toHaveCount(20);
 for(const checkbox of await cards.getByRole('checkbox').all())await checkbox.check();
 await cards.getByRole('button',{name:'Next programs',exact:true}).click();
 for(const checkbox of (await cards.getByRole('checkbox').all()).slice(0,2))await checkbox.check();
 await expect(page.getByTestId('open-comparison')).toContainText('(22)');await page.getByTestId('open-comparison').click();
 const comparison=page.getByTestId('program-comparison');await expect(comparison.getByText('Same whole-element composition',{exact:true})).toHaveCount(20);
 await comparison.getByRole('button',{name:'Next programs',exact:true}).click();await expect(comparison.getByText('Same whole-element composition',{exact:true})).toHaveCount(2);
 await page.getByTestId('close-comparison').click();await cards.getByRole('checkbox').nth(2).check();await cards.getByRole('checkbox').nth(3).check();await page.getByTestId('open-comparison').click();
 if(await comparison.getByRole('button',{name:'Next programs',exact:true}).isEnabled())await comparison.getByRole('button',{name:'Next programs',exact:true}).click();
 const duplicate=comparison.locator('[data-result-id="fixture-22"]'),compound=comparison.locator('[data-result-id="fixture-23"]');
 await expect(duplicate.getByText('1Lz+1Lo',{exact:true})).toHaveCount(2);await expect(compound.getByText('1Lo+1Lz',{exact:true})).toBeVisible();
 await duplicate.locator('summary').click();await expect(duplicate.locator('details').getByText('1T',{exact:true})).toHaveCount(1);
 await comparison.locator('[data-result-id="fixture-20"] summary').click();await expect(comparison.locator('[data-result-id="fixture-20"] details').getByText('1T',{exact:true})).toHaveCount(1);
 await expect(comparison.locator('[data-result-id="fixture-20"]>ol').getByText('1T',{exact:true})).toHaveCount(1);
});
