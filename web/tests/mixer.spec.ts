import {test,expect} from '@playwright/test';

test('one category-bounded slider provides Off, Basic and only eligible maxima',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 const sit=page.getByTestId('fader-SSp');await expect(sit).toBeEnabled();
 await expect(sit).toHaveAttribute('max','2'); // Bronze: Off, Basic, 1.
 await expect(page.locator('.mixer input[type=checkbox], .mixer select')).toHaveCount(0);
 await sit.press('Home');await expect(sit).toHaveAttribute('aria-valuetext','Off');
 await sit.press('ArrowUp');await expect(sit).toHaveAttribute('aria-valuetext','Maximum: Basic');
 await sit.press('End');await expect(sit).toHaveAttribute('aria-valuetext','Maximum: 1');
 await sit.press('ArrowUp');await expect(sit).toHaveAttribute('aria-valuetext','Maximum: 1');
 await page.getByLabel('Category').selectOption({label:'Adult Gold'});
 await expect(page.getByTestId('fader-T')).toHaveAttribute('max','2');
 await expect(page.getByTestId('fader-F')).toHaveAttribute('max','1');
 await expect(page.getByTestId('fader-StSq')).toHaveAttribute('max','3'); // Off, Basic, 1, 2.
 const choreo=page.getByTestId('fader-ChSp');await choreo.press('Home');await expect(choreo).toHaveAttribute('aria-valuetext','Off');
 await choreo.press('End');await expect(choreo).toHaveAttribute('aria-valuetext','Enabled');
});

test('category changes clamp the displayed ceiling without erasing saved personal maxima',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 await page.getByLabel('Category').selectOption({label:'Adult Silver'});
 const spin=page.getByTestId('fader-CLSp');await expect(spin).toHaveAttribute('max','3');await spin.press('Home');await spin.press('End');
 await expect(spin).toHaveAttribute('aria-valuetext','Maximum: 2');
 await page.getByLabel('Category').selectOption({label:'Adult Bronze'});
 await expect(spin).toHaveAttribute('max','2');await expect(spin).toHaveAttribute('aria-valuetext','Maximum: 1');
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('skating-c2-workspace')!).personalLimits.spins.CLSp?.maximum)).toBe(2);
 await page.reload();await expect(spin).toHaveAttribute('aria-valuetext','Maximum: 1');
 await page.getByLabel('Category').selectOption({label:'Adult Silver'});
 await expect(spin).toHaveAttribute('aria-valuetext','Maximum: 2');
});
