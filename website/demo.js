(() => {
  const $ = selector => document.querySelector(selector);
  const words = [...document.querySelectorAll('[data-word]')];
  const action = $('.demo-action');
  const motion = matchMedia('(prefers-reduced-motion: reduce)');
  const sounds = { fix: new Audio('sounds/ready.wav'), reroll: new Audio('sounds/rework.wav') };
  Object.values(sounds).forEach(sound => { sound.volume = 0.3; sound.preload = 'auto'; });
  const originals = ['pasghetti', 'delicious', 'definitley'];
  let stage = 'before', soundEnabled = false, paused = motion.matches;
  let timer, due = 0, remaining = 0, next;

  function cancel() { clearTimeout(timer); timer = undefined; }
  function schedule(callback, delay) {
    cancel(); next = callback; remaining = delay;
    if (!paused && !document.hidden) { due = performance.now() + remaining; timer = setTimeout(fire, remaining); }
  }
  function fire() { timer = undefined; const callback = next; next = undefined; callback?.(); }
  function suspend() { if (timer !== undefined) { remaining = Math.max(0, due - performance.now()); cancel(); } }
  function resume() { if (next && !paused && !document.hidden) { due = performance.now() + remaining; timer = setTimeout(fire, remaining); } }
  function play(name) {
    if (!soundEnabled || document.hidden) return;
    const audio = sounds[name]; audio.currentTime = 0;
    audio.play().catch(() => {});
  }
  function label(text, icon = '✦', busy = false) {
    $('.action-label').textContent = text; $('.action-icon').textContent = icon;
    action.disabled = busy; action.classList.toggle('busy', busy);
  }
  function announce(text) { $('.demo-announcement').textContent = text; }
  function reset() {
    cancel(); next = undefined; stage = 'before';
    words.forEach((word, i) => { TextMotion.setText(word, originals[i]); word.className = `word ${i === 1 ? 'reroll-word' : 'typo'}`; });
    $('.note').classList.remove('done');
    label('Fix typos'); announce('First, fix two typos.');
    schedule(fix, 2200);
  }
  function fix() {
    stage = 'fixing'; label('A little polish…', '✦', true);
    words.filter((_, i) => i !== 1).forEach(word => word.classList.add('chosen', 'working'));
    schedule(() => {
      words.filter((_, i) => i !== 1).forEach(word => { TextMotion.setText(word, word.dataset.word); word.className = 'word fixed landed'; });
      play('fix');
      announce('Spaghetti and definitely corrected.'); stage = 'fixed';
      label('Reroll a word', '⚄');
      schedule(select, 1900);
    }, motion.matches ? 0 : 850);
  }
  function select() {
    stage = 'selected'; words.forEach(word => word.classList.remove('fixed', 'landed'));
    words[1].classList.add('chosen');
    announce('Select delicious, then reroll just that word.');
    schedule(reroll, 1800);
  }
  function reroll() {
    stage = 'rerolling'; words[1].classList.add('chosen', 'working'); label('Finding the word…', '⚄', true);
    schedule(() => {
      TextMotion.setText(words[1], 'divine'); words[1].className = 'word fixed landed';
      play('reroll'); stage = 'done'; $('.note').classList.add('done');
      label('Once more?', '↻'); announce('Delicious becomes divine. Demo complete.');
    }, motion.matches ? 0 : 1000);
  }
  function enableSound() {
    if (soundEnabled) return;
    soundEnabled = true;
    // Unlock audio during the user's first demo interaction, including Safari.
    Object.values(sounds).forEach(audio => {
      audio.muted = true;
      audio.play()?.then(() => { audio.pause(); audio.currentTime = 0; audio.muted = false; })
        .catch(() => { audio.muted = false; });
    });
  }
  action.addEventListener('click', () => {
    enableSound();
    cancel(); next = undefined;
    // Manual steps also work with reduced motion enabled.
    if (stage === 'done') { reset(); return; }
    if (stage === 'before') fix();
    else if (stage === 'fixed') { select(); cancel(); next = undefined; reroll(); }
    else if (stage === 'selected') reroll();
    if (paused && next) fire();
  });
  document.addEventListener('visibilitychange', () => { if (document.hidden) { suspend(); Object.values(sounds).forEach(audio => audio.pause()); } else resume(); });
  motion.addEventListener('change', () => { paused = motion.matches; reset(); });
  reset();
})();
