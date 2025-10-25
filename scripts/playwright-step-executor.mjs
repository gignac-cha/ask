import { chromium } from 'playwright';
import fs from 'fs/promises';
import path from 'path';

/**
 * Step-by-step Playwright executor for automation loop
 * Executes a single action and returns current page state
 */

let globalBrowser = null;
let globalPage = null;

async function initBrowser() {
  if (!globalBrowser) {
    globalBrowser = await chromium.launch({
      headless: false,
      args: ['--disable-blink-features=AutomationControlled']
    });
    const context = await globalBrowser.newContext({
      userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
    });
    globalPage = await context.newPage();
  }
  return globalPage;
}

async function getSimplifiedDOM(page) {
  try {
    // Extract main text content, links, and form elements
    const domInfo = await page.evaluate(() => {
      const getVisibleText = (element) => {
        const style = window.getComputedStyle(element);
        if (style.display === 'none' || style.visibility === 'hidden') {
          return '';
        }
        return element.innerText || element.textContent || '';
      };

      const body = document.body;
      const mainText = getVisibleText(body).slice(0, 3000); // Limit to 3000 chars

      const links = Array.from(document.querySelectorAll('a[href]'))
        .slice(0, 20)
        .map(a => ({
          text: a.textContent?.trim().slice(0, 100),
          href: a.href
        }))
        .filter(l => l.text);

      const inputs = Array.from(document.querySelectorAll('input, textarea'))
        .slice(0, 10)
        .map(input => ({
          type: input.type || 'text',
          name: input.name,
          id: input.id,
          placeholder: input.placeholder
        }));

      const headings = Array.from(document.querySelectorAll('h1, h2, h3'))
        .slice(0, 10)
        .map(h => h.textContent?.trim())
        .filter(Boolean);

      return {
        title: document.title,
        url: window.location.href,
        mainText,
        headings,
        links,
        inputs
      };
    });

    return domInfo;
  } catch (error) {
    return {
      error: `Failed to extract DOM: ${error.message}`,
      url: await page.url().catch(() => 'unknown')
    };
  }
}

async function captureScreenshot(page) {
  try {
    const screenshot = await page.screenshot({
      type: 'png',
      fullPage: false // Only visible viewport
    });
    return screenshot.toString('base64');
  } catch (error) {
    console.error('Screenshot failed:', error.message);
    return null;
  }
}

async function executeStep(actionJson) {
  const action = JSON.parse(actionJson);
  const page = await initBrowser();

  const result = {
    success: false,
    action: action.function || action.action,
    timestamp: new Date().toISOString()
  };

  try {
    const funcName = action.function || action.action;
    const args = action.args || {};

    console.error(`[Executor] Executing: ${funcName}`, args);

    switch (funcName) {
      case 'goToURL':
        await page.goto(args.url, {
          waitUntil: 'domcontentloaded',
          timeout: 30000
        });
        result.success = true;
        result.message = `Navigated to ${args.url}`;
        break;

      case 'click':
        await page.click(args.selector, { timeout: 5000 });
        await page.waitForTimeout(1000); // Wait for any navigation/changes
        result.success = true;
        result.message = `Clicked ${args.selector}`;
        break;

      case 'typeText':
        await page.fill(args.selector, args.text, { timeout: 5000 });
        result.success = true;
        result.message = `Typed into ${args.selector}`;
        break;

      case 'getText':
        const text = await page.textContent(args.selector, { timeout: 5000 });
        result.success = true;
        result.message = `Got text from ${args.selector}`;
        result.extractedText = text;
        break;

      case 'wait':
        await page.waitForTimeout(args.ms || 1000);
        result.success = true;
        result.message = `Waited ${args.ms || 1000}ms`;
        break;

      case 'finish':
        result.success = true;
        result.message = 'Task completed';
        result.finalAnswer = args.answer || args.summary || 'Task completed successfully';
        break;

      default:
        result.success = false;
        result.error = `Unknown action: ${funcName}`;
    }

    // Always capture current page state (unless it's 'finish')
    if (funcName !== 'finish') {
      result.dom = await getSimplifiedDOM(page);
      result.screenshot = await captureScreenshot(page);
      result.currentUrl = await page.url();
    }

  } catch (error) {
    result.success = false;
    result.error = error.message;
    try {
      result.currentUrl = page.url();
    } catch (e) {
      result.currentUrl = 'unknown';
    }
  }

  return result;
}

async function cleanup() {
  if (globalBrowser) {
    await globalBrowser.close();
    globalBrowser = null;
    globalPage = null;
  }
}

// Main execution
const actionJson = process.argv[2];
const command = process.argv[3]; // Optional: 'cleanup' to close browser

if (command === 'cleanup') {
  await cleanup();
  console.log(JSON.stringify({ success: true, message: 'Browser closed' }));
  process.exit(0);
}

if (!actionJson) {
  console.error('Usage: node playwright-step-executor.mjs <action-json> [cleanup]');
  process.exit(1);
}

try {
  const result = await executeStep(actionJson);
  console.log(JSON.stringify(result));

  // Don't auto-cleanup - let Rust manage browser lifecycle
  process.exit(0);
} catch (error) {
  console.error(JSON.stringify({
    success: false,
    error: error.message,
    stack: error.stack
  }));
  process.exit(1);
}
