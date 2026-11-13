import type {Page} from '@playwright/test';
import {expect} from '@playwright/test';
export async function openPicker(page:Page){
 if(!await page.locator('.element-picker').evaluate((dialog:HTMLDialogElement)=>dialog.open))await page.getByTestId('open-element-picker').click();
}
export async function setEnabled(page:Page,id:string,enabled:boolean){
 const slider=page.getByTestId(`fader-${id}`);await expect(slider).toBeEnabled();
 await slider.press(enabled?'End':'Home');
 if(enabled)await expect(slider).not.toHaveValue('0');else await expect(slider).toHaveValue('0');
}
export async function setMaximum(page:Page,id:string,maximum:number){
 const slider=page.getByTestId(`fader-${id}`);await expect(slider).toBeEnabled();
 const wanted=`Maximum: ${maximum===0?'Basic':maximum}`;
 await slider.press('Home');
 for(let step=0;step<6&&await slider.getAttribute('aria-valuetext')!==wanted;step++)await slider.press('ArrowUp');
 await expect(slider).toHaveAttribute('aria-valuetext',wanted);
}
export async function configureSilver(page:Page){
 await page.getByLabel('Category').selectOption({label:'Adult Silver'});
 for(const kind of ['T','S','Lo','F','Lz','A'])await setMaximum(page,kind,1);
 for(const form of ['USp','SSp','CSp','LSp','CoSp','CUSp','CSSp','CCSp','CLSp','CCoSp','FUSp','FSSp','FCSp','FLSp','FCoSp','FCUSp','FCSSp','FCCSp','FCLSp','FCCoSp']){
  if(['USp','SSp','CSp','CCoSp'].includes(form))await setMaximum(page,form,['SSp','CCoSp'].includes(form)?1:0);
  else await setEnabled(page,form,false);
 }
 await setEnabled(page,'W',false);await setEnabled(page,'ChSp',false);
}
