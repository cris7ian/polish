// Fix the selected passage, then show several rerolls of one word.
(() => {
  const preview = document.querySelector('.app-preview');
  const passage = preview.querySelector('.app-demo-passage');
  const words = [...preview.querySelectorAll('[data-app-word]')];
  const originals = words.map(word => word.textContent);
  const target = preview.querySelector('[data-app-word="delicious"]');
  const fix = preview.querySelector('.app-demo-fix');
  const reroll = preview.querySelector('.app-demo-reroll');
  const motion = matchMedia('(prefers-reduced-motion: reduce)');
  let visible = false, timer, started = 0, remaining = 1400, step = 0;
  const beats = [
    () => passage.classList.add('selected'),
    () => { passage.classList.add('processing'); fix.classList.add('active'); },
    () => { words.forEach(word => { TextMotion.setText(word, word.dataset.appWord); }); passage.classList.remove('processing'); fix.classList.remove('active'); },
    () => { passage.classList.remove('selected'); target.classList.add('selected'); },
  ];
  const delays = [1000, 900, 2000, 1200];
  for (const alternative of ['divine', 'incredible', 'heavenly', 'unforgettable']) {
    beats.push(
      () => { target.classList.add('processing'); reroll.classList.add('active'); },
      () => { TextMotion.setText(target, alternative); target.classList.remove('processing'); reroll.classList.remove('active'); },
    );
    delays.push(900, 2000);
  }
  TextMotion.prepare(passage);
  function pause() {
    if (timer !== undefined) { clearTimeout(timer); timer = undefined; remaining = Math.max(0, remaining - (performance.now() - started)); }
  }
  function resume() {
    if (!visible || document.hidden || motion.matches || step >= beats.length || timer !== undefined) return;
    started = performance.now();
    timer = setTimeout(() => { timer = undefined; beats[step++](); remaining = delays[step - 1] ?? 0; resume(); }, remaining);
  }
  function reducedMotion() {
    pause();
    passage.classList.remove('processing', 'selected');
    words.forEach(word => { word.classList.remove('processing', 'selected'); TextMotion.setText(word, word.dataset.appWord); });
    fix.classList.remove('active'); reroll.classList.remove('active');
    if (motion.matches) step = beats.length;
    else { step = 0; remaining = 1400; originals.forEach((text, i) => { TextMotion.setText(words[i], text); }); resume(); }
  }
  new IntersectionObserver(entries => {
    visible = entries[0].isIntersecting && entries[0].intersectionRatio >= 0.5;
    if (visible) resume(); else pause();
  }, { threshold: 0.5 }).observe(preview);
  document.addEventListener('visibilitychange', () => { if (document.hidden) pause(); else resume(); });
  motion.addEventListener('change', reducedMotion);
  if (motion.matches) reducedMotion();
})();
