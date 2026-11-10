import {test,expect} from '@playwright/test';
test('explicit proof adopts a freshly assessed program and preserves retained identity',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 let creates=0;page.on('request',r=>{if(r.url().endsWith('/api/v1/searches')&&r.method()==='POST')creates++;});
 for(const code of ['CCSp1','CLSp1','ChSq1']){await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();await page.getByTestId('program-entry').last().getByRole('checkbox').check();}
 const ids=await page.getByTestId('program-entry').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('data-entry-id')));
 for(const kind of ['T','S','Lz'])await page.getByTestId(`enabled-${kind}`).uncheck();await page.getByTestId('enabled-W').uncheck();
 expect(creates).toBe(0);await page.getByTestId('calculate').click();
 await expect(page.getByTestId('search-maximum')).toContainText('10.30',{timeout:30000});await expect(page.getByTestId('adopt-result').first()).toBeEnabled();
 await page.getByTestId('adopt-result').first().click();await expect(page.getByTestId('assessment')).toContainText('Construction complies');await expect(page.getByTestId('assessment')).toContainText('10.30');
 const adopted=page.getByTestId('program-entry');for(const id of ids){await expect(page.locator(`[data-entry-id="${id}"]`).getByRole('checkbox')).toBeChecked();}
 expect(await adopted.count()).toBeGreaterThan(3);expect(await adopted.locator('input:checked').count()).toBe(3);
 await page.getByLabel('Language').selectOption('de');await page.getByLabel('Language').selectOption('ru');await page.getByLabel('Language').selectOption('en');await adopted.last().getByRole('button',{name:'Move up'}).click();expect(creates).toBe(1);await expect(page.getByTestId('search-stale')).toHaveCount(0);
 await page.getByTestId('maximum-F').selectOption('1');await expect(page.getByTestId('search-stale')).toBeVisible();await expect(page.getByTestId('adopt-result').first()).toBeDisabled();expect(creates).toBe(1);
});

