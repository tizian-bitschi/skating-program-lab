import {test,expect} from '@playwright/test';
test('workspace reload preserves entries, retention, limits and locale without a search',async({page})=>{
 let searches=0;page.on('request',r=>{if(r.url().includes('/api/v1/searches'))searches++;});
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await page.getByTestId('element-query').fill('1Lz');await page.getByTestId('add-1Lz').click();await page.getByTestId('program-entry').getByRole('checkbox').check();const id=await page.getByTestId('program-entry').getAttribute('data-entry-id');await page.getByTestId('maximum-T').selectOption('1');await page.getByLabel('Language').selectOption('de');await page.reload();await expect(page.getByTestId('program-entry')).toHaveAttribute('data-entry-id',id!);await expect(page.getByTestId('program-entry').getByRole('checkbox')).toBeChecked();await expect(page.getByLabel('Language')).toHaveValue('de');await expect(page.getByTestId('assessment')).toContainText('0,60');expect(searches).toBe(0);
});

test('corrupt snapshot stays intact until explicit replacement',async({page})=>{
 await page.addInitScript(()=>localStorage.setItem('skating-c2-workspace','{broken'));
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await expect(page.getByTestId('workspace-storage')).toContainText('corrupt');expect(await page.evaluate(()=>localStorage.getItem('skating-c2-workspace'))).toBe('{broken');await page.getByRole('button',{name:'Replace saved workspace with a new empty workspace',exact:true}).focus();await page.keyboard.press('Enter');await expect(page.getByTestId('workspace-storage')).toContainText('Workspace saved');await expect(page.getByTestId('program-entry')).toHaveCount(0);
});
