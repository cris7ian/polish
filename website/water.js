// Two superimposed travelling swells make a continuously changing waterline.
(() => {
  const paths = [...document.querySelectorAll('.wave path')];
  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)');
  let frame = 0;
  function draw(time) {
    for (let layer = 0; layer < paths.length; layer++) {
      let path = '';
      for (let x = 0; x <= 1440; x += 20) {
        const phase = time * (layer ? 0.00045 : -0.00032);
        // The SVG fills the panel: 200/400 keeps the flag split halfway down.
        const baseline = layer ? 204 : 196;
        const y = baseline + 14 * Math.sin(x / 155 + phase + layer) + 7 * Math.sin(x / 75 - phase * 1.3 + layer * 2);
        path += `${x ? 'L' : 'M'}${x} ${y.toFixed(1)} `;
      }
      paths[layer].setAttribute('d', `${path}L1440 400 L0 400 Z`);
    }
    if (!reduced.matches && !document.hidden) frame = requestAnimationFrame(draw);
  }
  function update() {
    cancelAnimationFrame(frame);
    draw(performance.now());
  }
  reduced.addEventListener('change', update);
  document.addEventListener('visibilitychange', update);
  update();
})();
