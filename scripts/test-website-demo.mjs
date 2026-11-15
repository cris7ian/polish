import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

function element() {
  const values = new Set();
  return {
    textContent: '', dataset: {}, hidden: false, disabled: false, listeners: {},
    firstElementChild: { textContent: '' },
    classList: { add: (...names) => names.forEach(n => values.add(n)), remove: (...names) => names.forEach(n => values.delete(n)), toggle: (n, on) => on ? values.add(n) : values.delete(n) },
    setAttribute() {}, addEventListener(name, fn) { this.listeners[name] = fn; },
  };
}
function setup(reduced = false) {
  const words = ['spaghetti', 'delicious', 'definitely'].map(word => Object.assign(element(), { dataset: { word } }));
  const elements = Object.fromEntries(['.demo-action', '.action-label', '.action-icon', '.demo-announcement', '.note'].map(s => [s, element()]));
  let pending; const audios = [];
  const document = {
    hidden: false, listeners: {},
    querySelector: selector => elements[selector],
    querySelectorAll: selector => selector === '[data-word]' ? words : [],
    addEventListener(name, fn) { this.listeners[name] = fn; },
  };
  runInNewContext(readFileSync('website/demo.js', 'utf8'), {
    TextMotion: { setText: (element, text) => { element.textContent = text; } },
    document, matchMedia: () => ({ matches: reduced, addEventListener() {} }),
    performance: { now: () => 0 },
    setTimeout: callback => { pending = callback; return 1; }, clearTimeout: () => { pending = undefined; },
    Audio: class { constructor() { this.plays = 0; audios.push(this); } play() { this.plays++; return Promise.resolve(); } pause() {} },
  });
  return { words, elements, document, audios,
    tick() { assert(pending, 'Expected next demo beat'); const fn = pending; pending = undefined; fn(); },
    hasTimer: () => Boolean(pending),
    click: selector => elements[selector].listeners.click(),
  };
}
const demo = setup();
assert.deepEqual(demo.words.map(w => w.textContent), ['pasghetti', 'delicious', 'definitley']);
demo.tick(); assert.equal(demo.elements['.demo-action'].disabled, true);
demo.tick(); assert.deepEqual(demo.words.map(w => w.textContent), ['spaghetti', 'delicious', 'definitely']);
demo.tick(); demo.tick(); demo.tick();
assert.deepEqual(demo.words.map(w => w.textContent), ['spaghetti', 'divine', 'definitely']);
assert.equal(demo.hasTimer(), false);
demo.click('.demo-action'); assert.equal(demo.words[0].textContent, 'pasghetti');
demo.document.hidden = true; demo.document.listeners.visibilitychange(); assert.equal(demo.hasTimer(), false);
demo.document.hidden = false; demo.document.listeners.visibilitychange(); assert.equal(demo.hasTimer(), true);
const quiet = setup(); for (let i = 0; i < 5; i++) quiet.tick();
assert(quiet.audios.every(a => a.plays === 0), 'Autoplay must remain silent');
const reduced = setup(true); assert.equal(reduced.hasTimer(), false);
reduced.click('.demo-action'); assert.equal(reduced.words[0].textContent, 'spaghetti');
reduced.click('.demo-action'); assert.equal(reduced.words[1].textContent, 'divine');
assert.equal(reduced.hasTimer(), false);
console.log('Demo checks passed: both edits, replay, visibility, silent autoplay, reduced motion.');
