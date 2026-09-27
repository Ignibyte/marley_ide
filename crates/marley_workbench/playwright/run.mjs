// Marley's Playwright runner (#523): runs a saved script on a Browser tab.
//
// Marley writes this file into its data folder and types `node run.mjs <script>` into a terminal
// beside the tab, with MARLEY_CDP_URL naming the tab's Chromium and MARLEY_TAB the tab's page.
// Playwright attaches to that Chromium over CDP, the tab's page goes to the script's default
// export, and at the end Playwright only lets go: the page, its context and the browser are the
// tab's, and stay open.
import { basename } from 'node:path';
import { pathToFileURL } from 'node:url';
import { chromium } from 'playwright-core';

const [script] = process.argv.slice(2);
const endpoint = process.env.MARLEY_CDP_URL;
const tab = process.env.MARLEY_TAB;
if (!script || !endpoint || !tab) {
  console.error('Marley runs this with a script, MARLEY_CDP_URL and MARLEY_TAB set.');
  process.exit(2);
}
const name = basename(script).replace(/\.mjs$/, '');

// The tab's page, by its target id; a page just attached can take a moment to show.
async function tabPage(context) {
  for (let tries = 0; tries < 50; tries += 1) {
    for (const page of context.pages()) {
      const session = await context.newCDPSession(page);
      const { targetInfo } = await session.send('Target.getTargetInfo');
      await session.detach();
      if (targetInfo.targetId === tab) {
        return page;
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`the Browser tab ${tab} is not in this Chromium`);
}

const started = Date.now();
let browser;
try {
  browser = await chromium.connectOverCDP(endpoint);
  const context = browser.contexts()[0];
  const page = await tabPage(context);
  const module = await import(pathToFileURL(script).href);
  if (typeof module.default !== 'function') {
    throw new Error(`${name} exports no default function`);
  }
  console.log(`▶ ${name} on ${page.url()}`);
  await module.default({ page, context, browser });
  console.log(`✓ ${name} passed in ${((Date.now() - started) / 1000).toFixed(1)} s`);
} catch (error) {
  console.error(`✗ ${name} failed: ${error?.stack ?? error}`);
  process.exitCode = 1;
} finally {
  // On a browser Playwright attached to, close() only disconnects.
  await browser?.close();
}
