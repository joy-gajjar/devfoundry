import { expect, test } from '@playwright/test'

test('preview surface shows bounded lifecycle and stop control', async ({ page }) => {
  await page.goto('/')
  await page.getByRole('tab', { name: 'Preview' }).click()
  await expect(page.getByRole('region', { name: 'Preview' })).toBeVisible()
  await expect(page.getByRole('status', { name: 'Preview status' })).toHaveText('ready')
  await expect(page.getByRole('button', { name: 'Stop preview' })).toBeVisible()
})
