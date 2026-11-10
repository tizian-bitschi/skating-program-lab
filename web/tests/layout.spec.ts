import {openPicker} from './picker';
import {test,expect} from '@playwright/test';

test('filled desktop keeps headings and primary controls visible in all locales',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 for(const code of ['1Lz','SSp1','ChSq1']){await openPicker(page);await page.getByTestId('element-query').fill(code);await page.getByTestId(`add-${code}`).click();}
 await page.getByText('Program is fully described',{exact:true}).click();
 await expect(page.getByTestId('assessment')).toContainText('Construction complies');
 for(const viewport of [{width:1280,height:720},{width:1440,height:900}]){
  await page.setViewportSize(viewport);
  for(const locale of ['en','de','ru']){
   await page.getByLabel('Language').selectOption(locale);
   for(const selector of ['.workspace>section>h2','[data-testid=calculate]','[data-testid=open-element-picker]','[data-testid=assessment]']){
    for(const control of await page.locator(selector).all()){
     const rect=await control.boundingBox();expect(rect,selector).not.toBeNull();expect(rect!.y,selector).toBeGreaterThanOrEqual(0);expect(rect!.y+rect!.height,selector).toBeLessThanOrEqual(viewport.height);
    }
   }
   expect(await page.locator('.workspace').evaluate(el=>el.scrollHeight<=el.clientHeight)).toBe(true);
  }
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
 await slider.focus();await page.keyboard.press('Home');await expect(page.getByTestId('enabled-SSp')).not.toBeChecked();
 await page.keyboard.press('ArrowUp');await expect(page.getByTestId('enabled-SSp')).toBeChecked();await expect(page.getByTestId('maximum-SSp')).toHaveValue('0');
 await expect(slider).toHaveAttribute('aria-valuetext','Maximum: Basic');
});

test('phone starts with discoverable program and calculation and exposes every control on request',async({page})=>{
 await page.setViewportSize({width:360,height:844});await page.goto('/');await page.getByLabel('Language').selectOption('en');
 await expect(page.getByTestId('calculate')).toBeInViewport();
 await page.getByRole('button',{name:'Show all controls'}).click();await expect(page.locator('[data-family=spin]')).toHaveCount(20);
 await page.getByRole('link',{name:'Program suggestions',exact:true}).click();await expect(page.getByTestId('calculate')).toBeInViewport();
});

test('Escape closes the picker even while a search query is filled and restores focus',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');await openPicker(page);await page.getByTestId('element-query').fill('Lutz');await page.keyboard.press('Escape');await expect(page.getByRole('dialog')).toHaveCount(0);await expect(page.getByTestId('open-element-picker')).toBeFocused();
});
