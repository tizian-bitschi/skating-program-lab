import {test,expect} from '@playwright/test';

test('synthetic 10000 HTTP records append without moving focus or losing selection',async({page})=>{
 test.setTimeout(60000);
 // Presentation workload only: these are explicitly synthetic, not computed optima.
 let revision=0,requestId='',delivered=0,release=()=>{};
 const held=new Promise<void>(resolve=>release=resolve);
 await page.route('**/api/v1/searches/identity',route=>route.fulfill({json:{creationNamespace:'synthetic-presentation'}}));
 await page.route('**/api/v1/searches',route=>{const body=route.request().postDataJSON();revision=body.revision;requestId=body.requestId;return route.fulfill({status:202,json:{id:'synthetic-presentation',accessToken:'fixture-only',requestId,revision}});});
 await page.route('**/api/v1/searches/synthetic-presentation/updates?*',async route=>{
  const after=Number(new URL(route.request().url()).searchParams.get('after'));
  if(after===100)await held;
  const cursor=Math.min(after+100,10000);delivered=cursor;
  const results=Array.from({length:cursor-after},(_,index)=>({id:`synthetic-${after+index+1}`,entries:[{elementKey:'synthetic-only',code:`SYNTHETIC ${after+index+1}`,requiredId:null}],assessment:{totalHundredths:100,elements:[{creditedHundredths:100,parts:[]}]}}));
  await route.fulfill({json:{id:'synthetic-presentation',requestId,revision,status:cursor===10000?'completed':'running',phase:cursor===10000?'terminal':'collecting',maximumBaseHundredths:100,confirmedCount:10000,cursor,results}});
 });
 await page.route('**/api/v1/searches/synthetic-presentation/cancel',route=>route.fulfill({json:{}}));
 await page.route('**/api/v1/searches/synthetic-presentation',route=>route.fulfill({json:{}}));
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await page.getByTestId('calculate').click();
 const list=page.locator('.planning-search>.result-browser');
 await expect(list.locator('.search-results>li')).toHaveCount(20);
 const checkbox=list.getByRole('checkbox').first();await checkbox.check();await checkbox.focus();
 const scroll=await list.locator('.search-results').evaluate(el=>{el.scrollTop=70;return el.scrollTop;});
 release();await expect.poll(()=>delivered).toBe(10000);
 await expect(page.locator('.planning-search>[role=status]')).toContainText('Confirmed programs: 10000');
 await expect(checkbox).toBeFocused();await expect(checkbox).toBeChecked();
 expect(await list.locator('.search-results').evaluate(el=>el.scrollTop)).toBe(scroll);
 await expect(list.locator('.search-results>li')).toHaveCount(20);
 await list.getByRole('spinbutton',{name:'Page'}).fill('500');await list.getByRole('spinbutton',{name:'Page'}).press('Enter');
 await expect(list.locator('.search-results>li').last()).toContainText('SYNTHETIC 10000');
 await list.getByRole('checkbox').last().check();
 await expect(page.getByTestId('program-comparison')).toContainText('SYNTHETIC 1');await expect(page.getByTestId('program-comparison')).toContainText('SYNTHETIC 10000');
 await list.getByRole('spinbutton',{name:'Page'}).fill('1');await list.getByRole('spinbutton',{name:'Page'}).press('Enter');await expect(list.getByRole('checkbox').first()).toBeChecked();
});
