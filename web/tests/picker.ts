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
