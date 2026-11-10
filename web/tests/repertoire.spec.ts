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


test('an independently registered policy uses the unchanged browser and HTTP path',async({page,request})=>{
 const origin='http://127.0.0.1:3156';
 const profilesResponse=await request.get(`${origin}/api/v1/profiles`);
 expect(profilesResponse.ok()).toBe(true);
 const profiles=await profilesResponse.json();
 const profile=profiles.find((p:{label:string})=>p.label==='Independent HTTP test policy');
 expect(profile).toBeDefined();
 await page.goto(origin);
 await page.getByLabel('Language').selectOption('en');
 await page.getByLabel('Category').selectOption(profile.context.profile);
 await expect(page.getByTestId('enabled-T')).toBeDisabled();
 await expect(page.getByTestId('enabled-Lz')).toBeEnabled();
 const codes=page.getByTestId('repertoire').locator('code');
 await expect(codes.filter({hasText:/^1Lz$/})).toHaveCount(1);
 await expect(codes.filter({hasText:/^1T$/})).toHaveCount(0);
 await expect(page.locator('[data-family="spin"]')).toHaveCount(20);
 await page.getByTestId('enabled-Lz').uncheck();
 await expect(codes.filter({hasText:/^1Lz$/})).toHaveCount(0);
 const response=await request.post(`${origin}/api/v1/repertoire`,{data:{requestId:'independent-browser-check',revision:3,context:profile.context,personalLimits:{}}});
 expect(response.status()).toBe(200);
 const body=await response.json();
 expect(body.elements.some((element:{code:string})=>element.code==='1Lz')).toBe(true);
 expect(body.elements.some((element:{code:string})=>element.code==='1T')).toBe(false);
});
