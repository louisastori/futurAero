import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./apps/desktop/e2e",
  timeout: 30_000,
  expect: {
    timeout: 10_000
  },
  fullyParallel: false,
  reporter: [["list"]],
  use: {
    baseURL: "http://127.0.0.1:1420",
    trace: "retain-on-failure",
    video: "retain-on-failure"
  },
  webServer: {
    command: "npm --workspace @futureaero/desktop run dev",
    url: "http://127.0.0.1:1420",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] }
    }
  ]
});
