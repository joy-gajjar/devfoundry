import { expect, test } from '@playwright/test'

test('desktop workspace navigates between chat, tasks, and docs', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('heading', { name: 'Session transcript' })).toBeVisible()
  await page.getByRole('tab', { name: 'Tasks' }).click()
  await expect(page.getByRole('heading', { name: 'Task board' })).toBeVisible()
  await expect(page.getByText('Review API contract')).toBeVisible()
  await page.getByRole('tab', { name: 'Docs' }).click()
  await expect(page.getByRole('heading', { name: 'Project documents' })).toBeVisible()
  await expect(page.getByText('Project README')).toBeVisible()
})

test('mobile workspace keeps status and tabs accessible', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto('/')
  await expect(page.getByRole('status')).toHaveText('Connected')
  await expect(page.getByRole('tab', { name: 'Chat' })).toHaveAttribute('aria-selected', 'true')
  await page.getByRole('tab', { name: 'Tasks' }).press('Enter')
  await expect(page.getByRole('heading', { name: 'Task board' })).toBeVisible()
})
