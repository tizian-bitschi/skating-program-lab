import {test,expect} from '@playwright/test';
test('original source edition link preserves exact reference',async({page})=>{
 await page.goto('/');await page.getByLabel('Language').selectOption('en');
 await page.locator('.sources summary').click();
 const reference=page.locator('.sources li').filter({hasText:'ISU Communication 2786'});
 await expect(reference).toContainText('2026-06-02');
 await expect(reference.getByRole('link',{name:/Open original source/})).toHaveAttribute('href','https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Transparency/ISU-Communications/2786-SinglesPairs-SOV-Update-260602-1780481099-6311.pdf');
 await page.getByLabel('Language').selectOption('de');await expect(reference.getByRole('link',{name:/Originalquelle öffnen/})).toBeVisible();
 await page.getByLabel('Language').selectOption('ru');await expect(reference.getByRole('link',{name:/Открыть оригинал источника/})).toBeVisible();
});
