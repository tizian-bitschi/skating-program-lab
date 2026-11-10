import { test, expect } from '@playwright/test';
test('category changes preserve explicit limits and all twenty spin controls', async ({ page }) => {
 await page.goto('/');
 await expect(page.getByTestId('form-CSp')).toBeVisible();
 await expect(page.locator('[data-family="spin"]')).toHaveCount(20);
 await page.getByLabel('Language').selectOption('en');
 await page.getByLabel('Category').selectOption({label:'Adult Silver'});
 await page.getByTestId('maximum-CSp').selectOption('0');
 await expect(page.getByTestId('maximum-CSp')).toHaveValue('0');
 await page.getByLabel('Category').selectOption({label:'Adult Bronze'});
 await expect(page.getByTestId('maximum-CSp')).toHaveValue('0');
 await expect(page.getByTestId('repertoire')).toContainText('CSpB');
 await expect(page.getByTestId('repertoire').locator('code').filter({hasText:/^CSp1$/})).toHaveCount(0);
 await page.getByTestId('enabled-CSp').uncheck();
 await expect(page.getByTestId('repertoire').locator('code').filter({hasText:/^CSpB$/})).toHaveCount(0);
 await page.getByLabel('Language').selectOption('ru');
 await expect(page.locator('html')).toHaveAttribute('lang','ru');
});



test('three languages reflow and keyboard controls work against the service', async ({page})=>{
 await page.goto('/');
 await expect(page.locator('[data-family="spin"]')).toHaveCount(20);
 for(const viewport of [{width:1280,height:720},{width:1440,height:900},{width:390,height:844},{width:320,height:900}]) {
  await page.setViewportSize(viewport);
  for(const locale of ['de','en','ru']) {
   await page.getByLabel('Language').selectOption(locale);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth)).toBe(true);
   if(viewport.width>=1100){expect(await page.locator('.sources').evaluate(el=>el.getBoundingClientRect().bottom),locale).toBeLessThanOrEqual(viewport.height);}
   await expect(page.getByTestId('form-CCSp')).toBeVisible();
   await expect(page.getByTestId('form-CLSp')).toBeVisible();
  }
 }
 await page.setViewportSize({width:1280,height:720});
 await page.getByLabel('Language').selectOption('en');
 await page.getByTestId('enabled-T').focus();
 await page.keyboard.press('Space');
 await expect(page.getByTestId('enabled-T')).not.toBeChecked();
 await expect(page.getByTestId('repertoire').locator('code').filter({hasText:/^1T$/})).toHaveCount(0);
 await page.screenshot({path:'test-results/repertoire-1280.png',fullPage:true});
});


test('every service-advertised built-in profile renders through the same UI',async({page,request})=>{
 const profiles=await (await request.get('/api/v1/profiles')).json();
 await page.goto('/');
 await page.getByLabel('Language').selectOption('en');
 for(const profile of profiles){
  if(await page.getByLabel('Category').inputValue()===profile.context.profile){continue;}
  const response=page.waitForResponse(r=>r.url().endsWith('/api/v1/repertoire')&&r.request().postDataJSON().context.profile===profile.context.profile);
  await page.getByLabel('Category').selectOption(profile.context.profile);
  expect((await response).ok()).toBe(true);
  await expect(page.locator('[data-family="spin"]')).toHaveCount(20);
 }
});

