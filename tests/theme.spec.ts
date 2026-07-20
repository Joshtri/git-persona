import { expect, test } from "@playwright/test";

const APP_URL = "http://localhost:1420";

test.describe("Theme switching", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(APP_URL);
    // Wait for the app shell to render
    await page.waitForSelector("[data-theme]", { timeout: 8000 });
  });

  test("starts in dark mode by default", async ({ page }) => {
    const theme = await page.evaluate(() => document.documentElement.dataset.theme);
    expect(theme).toBe("dark");
  });

  test("data-theme is on <html>, not an inner div", async ({ page }) => {
    const isOnHtml = await page.evaluate(() => document.documentElement.hasAttribute("data-theme"));
    const innerDivHasTheme = await page.evaluate(() => !!document.querySelector("div[data-theme]"));
    expect(isOnHtml).toBe(true);
    expect(innerDivHasTheme).toBe(false);
  });

  test("theme changes immediately when select changes — no Save required", async ({ page }) => {
    // Navigate to Settings via sidebar
    await page.click('button:has-text("Settings")');
    await page.waitForSelector("select", { timeout: 5000 });

    // Switch to Light
    await page.selectOption("select", "Light");
    const lightTheme = await page.evaluate(() => document.documentElement.dataset.theme);
    expect(lightTheme).toBe("light");

    // Switch to Dark
    await page.selectOption("select", "Dark");
    const darkTheme = await page.evaluate(() => document.documentElement.dataset.theme);
    expect(darkTheme).toBe("dark");

    // Switch to System — theme must be "dark" or "light" depending on OS
    await page.selectOption("select", "System");
    const systemTheme = await page.evaluate(() => document.documentElement.dataset.theme);
    expect(["dark", "light"]).toContain(systemTheme);
  });

  test("light mode applies correct CSS variable on <html>", async ({ page }) => {
    await page.click('button:has-text("Settings")');
    await page.waitForSelector("select");
    await page.selectOption("select", "Light");

    const bg = await page.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue("--color-bg").trim()
    );
    // Light mode --color-bg should be a light oklch value (high lightness)
    expect(bg).toContain("oklch");
    // Check it's not the dark default (oklch(10% ...))
    expect(bg).not.toMatch(/oklch\(\s*10%/);
  });

  test("dark mode applies correct CSS variable on <html>", async ({ page }) => {
    await page.click('button:has-text("Settings")');
    await page.waitForSelector("select");

    // First go light, then back to dark — tests the toggle
    await page.selectOption("select", "Light");
    await page.selectOption("select", "Dark");

    const bg = await page.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue("--color-bg").trim()
    );
    expect(bg).toContain("oklch");
    expect(bg).toMatch(/oklch\(\s*10%/);
  });

  test("portals inherit the theme — data-theme is on <html> so all body children get it", async ({
    page,
  }) => {
    // If data-theme is on <html>, every portal rendered into <body> automatically
    // inherits the CSS variables. Verify the architecture:
    // 1. <html> has data-theme
    // 2. <body> does NOT have data-theme (which would override or compete)
    // 3. No inner <div> has data-theme
    const results = await page.evaluate(() => ({
      htmlHasTheme: document.documentElement.hasAttribute("data-theme"),
      bodyHasTheme: document.body.hasAttribute("data-theme"),
      innerDivHasTheme: !!document.querySelector("div[data-theme]"),
      htmlTheme: document.documentElement.dataset.theme,
    }));

    expect(results.htmlHasTheme).toBe(true);
    expect(results.bodyHasTheme).toBe(false);
    expect(results.innerDivHasTheme).toBe(false);

    // Switch to light and confirm <html> carries it — portals into <body> inherit via cascade
    await page.click('button:has-text("Settings")');
    await page.selectOption("select", "Light");

    const lightResults = await page.evaluate(() => ({
      htmlTheme: document.documentElement.dataset.theme,
      bodyBg: getComputedStyle(document.body).backgroundColor,
    }));
    expect(lightResults.htmlTheme).toBe("light");
    // body's background should now be the light --color-bg value (not the dark one)
    // We verify the CSS variable resolved correctly on body
    const bodyBgVar = await page.evaluate(() =>
      getComputedStyle(document.body).getPropertyValue("--color-bg").trim()
    );
    expect(bodyBgVar).not.toMatch(/oklch\(\s*10%/);
  });
});
