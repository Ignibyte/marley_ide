// A Playwright script Marley runs on a Browser tab: the tab's `page`, its `context` and the
// `browser` it lives in, which Playwright reached over the tab's own Chromium. MARLEY_CDP_URL
// and MARLEY_TAB name that Chromium and the tab, for any tool this script starts.
//
// Leave the page, the context and the browser open, and the viewport as it is: they are the
// tab's. A dialog the script does not handle is dismissed, and a throw fails the run.
export default async function ({ page, context, browser }) {
  // For example:
  // await page.getByRole('button', { name: 'Sign in' }).click();
}
