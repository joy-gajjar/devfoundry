import { expect, test } from '@playwright/test'

test('desktop shell exposes keyboard-accessible theme and skip navigation', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 })
  await page.goto('/')

  await expect(page.locator('.workspace-grid')).toBeVisible()
  await expect(page.getByRole('link', { name: 'Skip to workspace' })).toBeAttached()
  await page.keyboard.press('Tab')
  await expect(page.getByRole('link', { name: 'Skip to workspace' })).toBeFocused()
  await page.keyboard.press('Enter')
  await expect(page.locator('#main-content')).toBeFocused()

  await page.getByRole('button', { name: 'Switch to light theme' }).click()
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light')
  await expect(page.getByRole('button', { name: 'Switch to dark theme' })).toBeVisible()
})

test('mobile surface navigation remains contained and keyboard reachable', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto('/')
  await page.getByRole('tab', { name: 'Settings' }).focus()
  await page.keyboard.press('Enter')
  await expect(page.getByRole('heading', { name: 'Notification settings' })).toBeVisible()

  const viewport = await page.evaluate(() => ({
    width: document.documentElement.scrollWidth,
    viewport: window.innerWidth,
  }))
  expect(viewport.width).toBeLessThanOrEqual(viewport.viewport)
})
