import { test, expect } from '@playwright/test'

test('workspace exposes worker and terminal projections without implying acceptance', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('tab', { name: 'Tasks' })).toBeVisible()
  await page.getByRole('tab', { name: 'Terminal' }).click()
  await expect(page.getByRole('region', { name: 'Native terminal' })).toContainText('No terminal output')
  await page.getByRole('tab', { name: 'Tasks' }).click()
  await expect(page.getByRole('button', { name: /Workspace migrationrunning/ })).toBeVisible()
  await expect(page.getByText('accepted')).not.toBeVisible()
})
