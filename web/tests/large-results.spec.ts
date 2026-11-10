import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('synthetic 10000 HTTP records append without moving focus or losing selection',async({page},testInfo)=>{
 test.setTimeout(60000);
 await page.addInitScript(()=>{
  const metrics:{batches:number[];latest?:{cursor:number;time:number}}={batches:[]};(window as unknown as {syntheticMetrics:typeof metrics}).syntheticMetrics=metrics;
  const original=Response.prototype.json;
  Response.prototype.json=async function(){const body=await original.call(this);if(body.id==='synthetic-presentation'&&body.results)metrics.latest={cursor:body.cursor,time:performance.now()};return body;};
  new MutationObserver(()=>{const batch=metrics.latest;if(batch&&document.querySelector('.planning-search>.result-browser .result-pages span')?.textContent?.includes(`/ ${batch.cursor} `)){metrics.latest=undefined;requestAnimationFrame(()=>metrics.batches.push(performance.now()-batch.time));}}).observe(document,{childList:true,subtree:true,characterData:true});
 });
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
 const cdp=await page.context().newCDPSession(page),memory:{count:number;cycle:number;usedSize:number}[]=[];
 async function sample(count:number,cycle=0){await cdp.send('HeapProfiler.collectGarbage');const heap=await cdp.send('Runtime.getHeapUsage');memory.push({count,cycle,usedSize:heap.usedSize});}
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await sample(0);await page.getByTestId('calculate').click();
 const list=page.locator('.planning-search>.result-browser');
 await expect(list.locator('.search-results>li')).toHaveCount(20);
 const checkbox=list.getByRole('checkbox').first();await checkbox.check();await checkbox.focus();
 await sample(100);
 const scroll=await list.locator('.search-results').evaluate(el=>{el.scrollTop=70;return el.scrollTop;});
 release();await expect.poll(()=>delivered).toBe(10000);
 await expect(page.locator('.planning-search>[role=status]')).toContainText('Confirmed programs: 10000');
 await expect(checkbox).toBeFocused();await expect(checkbox).toBeChecked();
 expect(await list.locator('.search-results').evaluate(el=>el.scrollTop)).toBe(scroll);
 await expect(list.locator('.search-results>li')).toHaveCount(20);
 await sample(10000);
 await list.getByRole('spinbutton',{name:'Page'}).fill('500');await list.getByRole('spinbutton',{name:'Page'}).press('Enter');
 await expect(list.locator('.search-results>li').last()).toContainText('SYNTHETIC 10000');
 await list.getByRole('checkbox').last().check();
 await expect(page.getByTestId('program-comparison')).toContainText('SYNTHETIC 1');await expect(page.getByTestId('program-comparison')).toContainText('SYNTHETIC 10000');
 await list.getByRole('spinbutton',{name:'Page'}).fill('1');await list.getByRole('spinbutton',{name:'Page'}).press('Enter');await expect(list.getByRole('checkbox').first()).toBeChecked();
 const batches=await page.evaluate(()=>(window as unknown as {syntheticMetrics:{batches:number[]}}).syntheticMetrics.batches);expect(batches.length).toBeGreaterThan(0);console.log(`synthetic batch presentation ms: ${JSON.stringify(batches)}`);expect(Math.max(...batches)).toBeLessThanOrEqual(200);
 for(let cycle=1;cycle<=3;cycle++){await page.reload();await expect(page.getByTestId('calculate')).toBeEnabled();await sample(0,cycle);delivered=0;await page.getByTestId('calculate').click();await expect.poll(()=>delivered).toBe(10000);await expect(list.locator('.result-pages')).toContainText('10000');await sample(10000,cycle);}
 await page.reload();await expect(page.getByTestId('calculate')).toBeEnabled();await sample(0,4);
 const observation=JSON.stringify({runtime:await page.evaluate(()=>navigator.userAgent),batches,memory,scope:'Clearly synthetic UI/HTTP data; CDP heap after explicit GC; no universal memory bound or leak claim'},null,2);
 await writeFile(testInfo.outputPath('synthetic-resource-observations.json'),observation);
 await testInfo.attach('synthetic-resource-observations',{body:observation,contentType:'application/json'});
});
