import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

const html = readFileSync('website/index.html', 'utf8');
assert.match(html, /brew install --cask cris7ian\/tap\/polish/);
assert.match(html, /Apple silicon · macOS 13\+/);
assert.match(html, /aria-label="Copy Homebrew install command"/);

async function check(clipboard, expected, success) {
  let click, timer;
  const classes = () => { const set = new Set(); return { add: x => set.add(x), remove: x => set.delete(x), contains: x => set.has(x) }; };
  const button = { addEventListener: (_, fn) => { click = fn; } };
  const command = { textContent: 'brew install --cask cris7ian/tap/polish' };
  const status = { textContent: '', classList: classes() };
  const container = { classList: classes(), offsetWidth: 1 };
  const elements = { '.copy-command': button, '#brew-command': command, '#brew-copy-status': status, '.brew-install': container };
  runInNewContext(readFileSync('website/homebrew-copy.js', 'utf8'), {
    document: { querySelector: selector => elements[selector] }, navigator: { clipboard },
    clearTimeout: () => {}, setTimeout: fn => { timer = fn; return 1; },
  });
  await click();
  assert.equal(status.textContent, expected);
  assert.equal(container.classList.contains('copy-success'), success);
  assert.equal(status.classList.contains('copy-error'), !success);
  if (success) { timer(); assert.equal(container.classList.contains('copy-success'), false); }
}
let copied;
await check({ writeText: async text => { copied = text; } }, 'Homebrew install command copied.', true);
assert.equal(copied, 'brew install --cask cris7ian/tap/polish');
await check({ writeText: async () => { throw new Error('denied'); } }, 'Could not copy. Select the command to copy it.', false);
await check(undefined, 'Could not copy. Select the command to copy it.', false);
console.log('Homebrew copy checks passed.');
