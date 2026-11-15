// Match Rune's selected-glyph wave: sin(time * 5 - byte * .38) * 1.6.
(() => {
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  function prepare(root) {
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    const nodes = [];
    while (walker.nextNode()) nodes.push(walker.currentNode);
    let index = 0;
    for (const node of nodes) {
      if (node.parentElement.closest('.motion-glyph')) continue;
      const fragment = document.createDocumentFragment();
      for (const token of node.textContent.split(/(\s+)/)) {
        if (!token) continue;
        if (/^\s+$/.test(token)) { fragment.append(document.createTextNode(token)); index += new TextEncoder().encode(token).length; continue; }
        const run = document.createElement('span'); run.className = 'glyph-run';
        for (const letter of token) {
          const glyph = document.createElement('span'); glyph.className = 'motion-glyph';
          glyph.textContent = letter; glyph.dataset.phase = index; glyph.style.setProperty('--glyph-index', index);
          index += new TextEncoder().encode(letter).length; run.append(glyph);
        }
        fragment.append(run);
      }
      node.replaceWith(fragment);
    }
  }
  function setText(root, text) { root.textContent = text; prepare(root); }
  window.TextMotion = { prepare, setText };
  let frame, previous = [];
  function draw(now) {
    previous.forEach(glyph => { glyph.style.transform = ''; });
    previous = [];
    if (!document.hidden && !reduced.matches) {
      for (const target of document.querySelectorAll('.working, .processing')) {
        const hero = Boolean(target.closest('.art'));
        const amplitude = parseFloat(getComputedStyle(target).fontSize) * (hero ? 0.12 : 1.6 / 15);
        for (const glyph of target.querySelectorAll('.motion-glyph')) {
          const y = Math.sin(now * 0.005 - Number(glyph.dataset.phase) * 0.38) * amplitude;
          glyph.style.transform = `translateY(${y.toFixed(3)}px)`;
          previous.push(glyph);
        }
      }
      frame = requestAnimationFrame(draw);
    }
  }
  function refresh() { cancelAnimationFrame(frame); draw(performance.now()); }
  document.addEventListener('visibilitychange', refresh);
  reduced.addEventListener('change', refresh);
  refresh();
})();
