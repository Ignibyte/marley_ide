// Marley's Playwright runner (#523): runs a saved script on a Browser tab.
//
// Marley writes this file into its data folder and types `node run.mjs <script>` into a terminal
// beside the tab, with MARLEY_CDP_FILE naming the file where the tab's Chromium relay keeps its
// address and token (#583) and MARLEY_TAB the tab's page. Playwright attaches to that Chromium
// over CDP with the token, the tab's page goes to the script's default export, and at the end
// Playwright only lets go: the page, its context and the browser are the tab's, and stay open.
// The token never goes on the command line: this file reads it, and hands MARLEY_CDP_URL and
// MARLEY_CDP_TOKEN to what the script starts.
import { readFileSync } from 'node:fs';
import { basename } from 'node:path';
import { pathToFileURL } from 'node:url';
import { chromium } from 'playwright-core';

const [script] = process.argv.slice(2);
const endpointFile = process.env.MARLEY_CDP_FILE;
const tab = process.env.MARLEY_TAB;
if (!script || !endpointFile || !tab) {
  console.error('Marley runs this with a script, MARLEY_CDP_FILE and MARLEY_TAB set.');
  process.exit(2);
}
let endpoint;
try {
  endpoint = JSON.parse(readFileSync(endpointFile, 'utf8'));
} catch (error) {
  console.error(`The browser's relay file cannot be read: ${error.message}. Is the browser running?`);
  process.exit(2);
}
process.env.MARLEY_CDP_URL = endpoint.url;
process.env.MARLEY_CDP_TOKEN = endpoint.token;
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
  browser = await chromium.connectOverCDP(endpoint.url, {
    headers: { Authorization: `Bearer ${endpoint.token}` },
  });
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
