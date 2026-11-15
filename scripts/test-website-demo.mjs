import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

const element = () => ({ hidden: false, classList: { values: new Set(), toggle(name, on) { if (on) this.values.add(name); else this.values.delete(name); }, remove(name) { this.values.delete(name); } } });
const versions = Array.from({ length: 4 }, element);
const idea = element();
const actions = Object.fromEntries(['polish', 'rework', 'shorten', 'idea', 'undo', 'redo', 'copy'].map(name => [name, element()]));
const motion = { matches: false, addEventListener() {} };
let draw;
const document = {
  hidden: false,
  querySelectorAll: () => versions,
  querySelector: selector => selector === '.demo-idea' ? idea : actions[selector.replace('.demo-press-', '')],
  addEventListener() {},
};
runInNewContext(readFileSync('website/demo.js', 'utf8'), {
  document, matchMedia: () => motion, performance: { now: () => 0 },
  requestAnimationFrame: callback => { draw = callback; return 1; }, cancelAnimationFrame() {},
});
const frame = t => draw(t);
const visible = () => versions.findIndex(version => !version.hidden);
assert.equal(visible(), 0);
for (const [time, version, action] of [[3000,0,'polish'],[9000,1,'rework'],[15000,2,'shorten'],[22500,3,'idea']]) {
  frame(time);
  assert.equal(visible(), version);
  assert(versions[version].classList.values.has('selected'));
  assert(versions[version].classList.values.has('processing'));
  assert(actions[action].classList.values.has('active'));
}
frame(28000); assert.equal(idea.hidden, false);
frame(31600); assert.equal(idea.hidden, true); assert(actions.undo.classList.values.has('active'));
frame(34600); assert.equal(idea.hidden, false); assert(actions.redo.classList.values.has('active'));
frame(37100); assert(actions.copy.classList.values.has('active'));
motion.matches = true; frame(38000);
assert.equal(visible(), 1); assert.equal(idea.hidden, true);
assert(!versions[1].classList.values.has('processing'));
console.log('Website demo checks passed');
