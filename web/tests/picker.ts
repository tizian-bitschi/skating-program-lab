import type {Page} from '@playwright/test';
export async function openPicker(page:Page){
 if(!await page.locator('.element-picker').evaluate((dialog:HTMLDialogElement)=>dialog.open))await page.getByTestId('open-element-picker').click();
}
