import {openPicker} from './picker';
import {test,expect} from '@playwright/test';

test('desktop uses readable page flow with related program areas side by side',async({page})=>{
 await page.setViewportSize({width:1280,height:800});await page.goto('/');await page.getByLabel('Language').selectOption('en');
 for(const code of ['1Lz','SSp1','ChSq1']){await openPicker(page);await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();}
 await page.getByText('Program is fully described',{exact:true}).click();await expect(page.getByTestId('assessment')).toContainText('Construction complies');
 for(const locale of ['en','de','ru']){
  await page.getByLabel('Language').selectOption(locale);
  const program=(await page.locator('#area-program').boundingBox())!,suggestions=(await page.locator('#area-suggestions').boundingBox())!;
  expect(program.y).toBe(suggestions.y);expect(program.x+program.width).toBeLessThan(suggestions.x);
  for(const selector of ['.program-list','[data-testid=assessment]','[data-testid=connection-exclusions]'])expect(await page.locator(selector).evaluate(el=>el.scrollHeight<=el.clientHeight)).toBe(true);
  await page.locator('#area-exclusions').scrollIntoViewIfNeeded();await expect(page.getByTestId('connection-exclusions')).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }
});

test('fader pointer preview stays local and keyboard preserves Off versus Basic',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 const slider=page.getByTestId('fader-SSp');await expect(slider).toBeEnabled();
 let requests=0;page.on('request',request=>{if(request.method()==='POST'&&/repertoire|assessments/.test(request.url()))requests++;});
 await page.waitForTimeout(200);requests=0;
 const rect=(await slider.boundingBox())!;
 await page.mouse.move(rect.x+rect.width/2,rect.y+5);await page.mouse.down();await page.mouse.move(rect.x+rect.width/2,rect.y+rect.height-20,{steps:4});
 await page.waitForTimeout(150);expect(requests).toBe(0);
 await page.mouse.up();await expect.poll(()=>requests).toBeGreaterThan(0);
 await slider.focus();await page.keyboard.press('Home');await expect(page.getByTestId('fader-SSp')).toHaveValue('0');
 await page.keyboard.press('ArrowUp');await expect(page.getByTestId('fader-SSp')).not.toHaveValue('0');await expect(page.getByTestId('fader-SSp')).toHaveAttribute('aria-valuetext','Maximum: Basic');
 await expect(slider).toHaveAttribute('aria-valuetext','Maximum: Basic');
});

test('phone navigation exposes full panels with keyboard focus and bottom clearance',async({page})=>{
 await page.setViewportSize({width:390,height:844});await page.goto('/');await page.getByLabel('Language').selectOption('en');
 await expect(page.getByTestId('discover-programs')).toBeInViewport();await expect(page.getByTestId('specify-elements')).toBeInViewport();
 await expect(page.locator('[data-family=spin]')).toHaveCount(20);
 for(const panel of ['program','suggestions','exclusions','ability']){
  const button=page.getByTestId(`panel-${panel}`);await button.focus();await page.keyboard.press('Enter');
  await expect(button).toHaveAttribute('aria-current','page');await expect(button).toBeFocused();
  await expect(page.locator(`#area-${panel}`)).toBeVisible();
  expect(await button.evaluate(el=>getComputedStyle(el).outlineStyle)).toBe('solid');
 }
});

test('Escape closes the picker even while a search query is filled and restores focus',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await openPicker(page);await page.getByTestId('element-query').fill('Lutz');await page.keyboard.press('Escape');await expect(page.getByRole('dialog')).toHaveCount(0);await expect(page.getByTestId('open-element-picker')).toBeFocused();
});
