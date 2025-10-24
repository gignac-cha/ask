import { chromium } from 'playwright';

/**
 * Playwright executor script
 * Executes browser actions based on JSON input
 */

async function executeActions(actionsJson) {
  const actions = JSON.parse(actionsJson);

  const browser = await chromium.launch({
    headless: true
  });

  const context = await browser.newContext();
  const page = await context.newPage();

  const results = [];

  try {
    for (const action of actions) {
      const { function: funcName, args } = action;

      console.log(`Executing: ${funcName}`, args);

      switch (funcName) {
        case 'goToURL':
          await page.goto(args.url, { waitUntil: 'networkidle' });
          results.push({
            action: funcName,
            success: true,
            result: `Navigated to ${args.url}`
          });
          break;

        case 'click':
          await page.click(args.selector);
          results.push({
            action: funcName,
            success: true,
            result: `Clicked ${args.selector}`
          });
          break;

        case 'getText':
          const text = await page.textContent(args.selector);
          results.push({
            action: funcName,
            success: true,
            result: text
          });
          break;

        case 'typeText':
          await page.fill(args.selector, args.text);
          results.push({
            action: funcName,
            success: true,
            result: `Typed into ${args.selector}`
          });
          break;

        default:
          results.push({
            action: funcName,
            success: false,
            error: `Unknown action: ${funcName}`
          });
      }

      // Small delay between actions
      await page.waitForTimeout(500);
    }
  } catch (error) {
    results.push({
      success: false,
      error: error.message
    });
  } finally {
    await browser.close();
  }

  return results;
}

// Read actions from command line argument
const actionsJson = process.argv[2];

if (!actionsJson) {
  console.error('Usage: node playwright-executor.mjs <actions-json>');
  process.exit(1);
}

try {
  const results = await executeActions(actionsJson);
  console.log(JSON.stringify(results, null, 2));
} catch (error) {
  console.error('Error:', error.message);
  process.exit(1);
}
