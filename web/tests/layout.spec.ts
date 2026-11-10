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
