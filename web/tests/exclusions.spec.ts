import {openPicker} from './picker';
import {test,expect} from '@playwright/test';
test('exact exclusion preserves pair components and longer wholes, survives reload and unavailable limits',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await page.getByLabel('Category').selectOption({label:'Adult Gold'});
 const exclusions=page.getByTestId('connection-exclusions');await exclusions.getByText('Excluded connections',{exact:true}).click();
 await exclusions.getByRole('button',{name:'Next part: 1Lz',exact:true}).click();await exclusions.getByRole('button',{name:'Next part: 1Lo',exact:true}).click();
 await exclusions.getByRole('button',{name:'Exclude connection 1Lz+1Lo',exact:true}).focus();await page.keyboard.press('Enter');
 await expect(exclusions.locator('[data-restore]')).toHaveCount(1);await expect(exclusions.locator('[data-restore]')).toBeFocused();await expect(exclusions.getByRole('button',{name:'Next part: 1Lo',exact:true})).toBeVisible();
 await exclusions.getByRole('button',{name:'Next part: 1Lo',exact:true}).click();await expect(exclusions.getByRole('button',{name:'Exclude connection 1Lz+1Lo+1Lo',exact:true})).toBeVisible();
 await openPicker(page);await page.getByTestId('element-query').fill('1Lz');await expect(page.getByTestId('add-1Lz')).toBeVisible();await openPicker(page);await page.getByTestId('element-query').fill('1Lo');await expect(page.getByTestId('add-1Lo')).toBeVisible();await page.getByTestId('close-element-picker').click();
 await page.getByTestId('enabled-Lo').uncheck();await page.getByLabel('Category').selectOption({label:'Adult Bronze'});await page.reload();await expect(exclusions.locator('[data-restore]')).toHaveCount(1);
 await exclusions.locator('[data-restore]').focus();await page.keyboard.press('Enter');await expect(exclusions.locator('[data-restore]')).toHaveCount(0);await expect(exclusions.locator('.connection-builder>summary')).toBeFocused();
 await page.reload();await expect(exclusions.locator('[data-restore]')).toHaveCount(0);
});

test('retained connection conflict is localized and restoration recovers a genuine calculation',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 for(const code of ['CCSp1','CLSp1','ChSq1']){await openPicker(page);await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();await page.getByTestId('program-entry').last().getByRole('checkbox').check();}
 for(const kind of ['T','S','Lz','W'])await page.getByTestId(`enabled-${kind}`).uncheck();
 await openPicker(page);const builder=page.getByTestId('program-connection-builder');await builder.getByText('Build a connection',{exact:true}).click();await builder.getByRole('button',{name:'Next part: 1F',exact:true}).click();await builder.getByRole('button',{name:'Next part: 1Lo',exact:true}).click();await builder.getByRole('button',{name:'Add connection 1F+1Lo',exact:true}).click();await page.getByTestId('program-entry').last().getByRole('checkbox').check();
 const exclusions=page.getByTestId('connection-exclusions');await exclusions.getByText('Excluded connections',{exact:true}).click();await exclusions.getByRole('button',{name:'Next part: 1F',exact:true}).click();await exclusions.getByRole('button',{name:'Next part: 1Lo',exact:true}).click();
 await page.getByTestId('calculate').click();await expect(page.getByTestId('search-maximum')).toContainText('10.30',{timeout:30000});
 let mutations=0;page.on('request',r=>{if(r.url().match(/\/api\/v1\/searches(?:\/[^/]+(?:\/cancel)?)?$/)&&r.method()!=='GET')mutations++;});
 await exclusions.getByRole('searchbox').fill('Loop');await page.getByLabel('Language').selectOption('de');await page.getByLabel('Language').selectOption('en');await expect(page.getByTestId('search-stale')).toHaveCount(0);expect(mutations).toBe(0);
 const start=Date.now();await exclusions.getByRole('button',{name:'Exclude connection 1F+1Lo',exact:true}).click();await expect(page.getByTestId('search-stale')).toBeVisible();await expect(page.getByTestId('assessment')).toContainText('Personal conflicts');console.log(`exclusion to conflict: ${Date.now()-start}ms`);
 for(const [locale,message] of [['de','Diese ganze Verbindung ist ausgeschlossen'],['ru','Это соединение исключено'],['en','This whole connection is excluded']]){await page.getByLabel('Language').selectOption(locale!);const details=page.getByTestId('assessment').locator('details').filter({hasText:message!});await details.locator('summary').first().click();await expect(details).toContainText('1F+1Lo');}
 await page.getByTestId('calculate').click();await expect(page.locator('.planning-search')).toContainText('Check the retained elements and planning restrictions.',{timeout:30000});await expect(page.locator('.planning-search')).toContainText('Retained: 1F+1Lo');
 await exclusions.locator('[data-restore]').click();await expect(page.getByTestId('assessment')).not.toContainText('Personal conflicts');await page.getByTestId('calculate').click();await expect(page.getByTestId('search-maximum')).toContainText('10.30',{timeout:30000});await expect(page.getByTestId('adopt-result').first()).toBeEnabled();
});

test('sequence keys, change-from-position controls and translated touch-sized restore survive categories',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await page.getByLabel('Category').selectOption({label:'Adult Gold'});const exclusions=page.getByTestId('connection-exclusions');await exclusions.getByText('Excluded connections',{exact:true}).click();await exclusions.getByRole('button',{name:'Next part: 1Lz',exact:true}).click();await exclusions.getByRole('button',{name:'Next part: 1A',exact:true}).click();await exclusions.getByRole('button',{name:'Exclude connection 1Lz+1A+SEQ',exact:true}).click();
 const keys=await page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!).excludedConnections);expect(keys).toHaveLength(1);expect(keys[0]).toMatch(/:1Lz\+1A\+SEQ:true$/);
 await exclusions.getByRole('button',{name:'Change from this part: 2 (1A)',exact:true}).click();await expect(exclusions.getByRole('button',{name:'Next part: 1Lo',exact:true})).toBeVisible();await exclusions.getByRole('button',{name:'Next part: 1Lo',exact:true}).click();await exclusions.getByRole('button',{name:'Exclude connection 1Lz+1Lo',exact:true}).click();await expect(exclusions.locator('[data-restore]')).toHaveCount(2);
 await page.getByLabel('Category').selectOption({label:'Adult Silver'});await exclusions.getByRole('button',{name:'Restore connection 1Lz+1Lo',exact:true}).last().click();await expect(exclusions.locator('[data-restore]')).toHaveCount(1);
 for(const locale of ['de','ru','en']){await page.getByLabel('Language').selectOption(locale);for(const width of [1280,390,320]){await page.setViewportSize({width,height:900});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);const box=await exclusions.locator('[data-restore]').boundingBox();expect(box!.height).toBeGreaterThanOrEqual(44);}}
 await page.reload();await expect(exclusions.locator('[data-restore]')).toContainText('1Lz+1A+SEQ');await exclusions.locator('[data-restore]').click();await expect(exclusions.locator('[data-restore]')).toHaveCount(0);
});

test('committing an exclusion invalidates in-flight creation while catalog search stays presentation-only',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');const exclusions=page.getByTestId('connection-exclusions');await exclusions.getByText('Excluded connections',{exact:true}).click();await exclusions.getByRole('button',{name:'Next part: 1F',exact:true}).click();await exclusions.getByRole('button',{name:'Next part: 1Lo',exact:true}).click();
 let release=()=>{},seen=()=>{};const held=new Promise<void>(r=>release=r),captured=new Promise<void>(r=>seen=r);let owner:{id:string;accessToken:string}|undefined,creates=0;
 await page.route('**/api/v1/searches',async route=>{creates++;const response=await route.fetch();owner=await response.json();seen();await held;await route.fulfill({response});});
 await page.getByTestId('calculate').click();await captured;await exclusions.getByRole('searchbox').fill('Loop');await page.getByLabel('Language').selectOption('de');await expect(page.getByTestId('calculate')).toBeDisabled();await page.getByLabel('Language').selectOption('en');await exclusions.getByRole('button',{name:'Exclude connection 1F+1Lo',exact:true}).click();release();await expect(page.getByTestId('calculate')).toBeEnabled();
 await expect.poll(async()=>{const response=await page.request.get(`/api/v1/searches/${owner!.id}/updates`,{headers:{Authorization:`Bearer ${owner!.accessToken}`}});return response.status();}).toBe(404);expect(creates).toBe(1);await expect(page.getByTestId('adopt-result')).toHaveCount(0);await expect(page.getByTestId('search-maximum')).toHaveCount(0);
});
