import { expect, test } from "@playwright/test";

test("administrator can sign in and open dashboard", async ({ page }) => {
  page.on("pageerror", (error) => console.log(`PAGE ERROR: ${error.stack}`));
  page.on("console", (message) => {
    if (message.type() === "error") console.log(`CONSOLE ERROR: ${message.text()}`);
  });
  await page.goto("/admin");
  await page.getByRole("button", { name: "Sign in as admin" }).click();
  await page.getByLabel("Email address").fill("admin@claudia.local");
  await page.getByLabel("Password").fill("ClaudiaAdmin#2026");
  await page.getByRole("button", { name: "Sign in", exact: true }).click();

  await expect(page.getByRole("heading", { name: "Overview" })).toBeVisible();
  await expect(page.getByText("Gross revenue")).toBeVisible();
  await expect(page.getByRole("button", { name: "products" })).toBeVisible();
});
