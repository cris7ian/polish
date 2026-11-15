import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

const html = readFileSync('website/index.html', 'utf8');
assert.match(html, /brew install --cask cris7ian\/tap\/polish/);
assert.match(html, /Apple silicon · macOS 13\+/);
assert.match(html, /aria-label="Copy Homebrew install command"/);

async function check(clipboard, expected) {
  let click;
  const button = { addEventListener: (_, fn) => { click = fn; } };
  const command = { textContent: 'brew install --cask cris7ian/tap/polish' };
  const status = { textContent: '' };
  const elements = { '.copy-command': button, '#brew-command': command, '#brew-copy-status': status };
  runInNewContext(readFileSync('website/homebrew-copy.js', 'utf8'), {
    document: { querySelector: selector => elements[selector] }, navigator: { clipboard },
  });
  await click();
  assert.equal(status.textContent, expected);
}
let copied;
await check({ writeText: async text => { copied = text; } }, 'Homebrew install command copied.');
assert.equal(copied, 'brew install --cask cris7ian/tap/polish');
await check({ writeText: async () => { throw new Error('denied'); } }, 'Could not copy. Select the command to copy it.');
await check(undefined, 'Could not copy. Select the command to copy it.');
console.log('Homebrew copy checks passed.');
