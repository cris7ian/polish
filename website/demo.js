// Each frame renders one coherent state of the demonstration.
(() => {
  const versions = [...document.querySelectorAll('.demo-version')];
  const idea = document.querySelector('.demo-idea');
  const buttons = {polish:document.querySelector('.demo-press-polish'), rework:document.querySelector('.demo-press-rework'), shorten:document.querySelector('.demo-press-shorten'), idea:document.querySelector('.demo-press-idea'), undo:document.querySelector('.demo-press-undo'), redo:document.querySelector('.demo-press-redo'), copy:document.querySelector('.demo-press-copy')};
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  let start = performance.now(), frame;
  function draw(now) {
    if (reduced.matches) {
      versions.forEach((v,i) => {v.hidden=i!==1;v.classList.remove('selected','processing')});
      idea.hidden=true; Object.values(buttons).forEach(b=>b.classList.remove('active')); return;
    }
    const t=(now-start)%39000;
    const index=t<6000?0:t<12000?1:t<18000?2:3;
    const selected=(t>=1500&&t<6000)||(t>=7500&&t<12000)||(t>=13500&&t<18000)||(t>=19500&&t<27000);
    const processing=(t>=2800&&t<6000)||(t>=8800&&t<12000)||(t>=14800&&t<18000)||(t>=22000&&t<27000);
    versions.forEach((v,i)=>{v.hidden=i!==index;v.classList.toggle('selected',i===index&&selected);v.classList.toggle('processing',i===index&&processing)});
    idea.hidden=t<27000 || (t>=31500&&t<34500);
    const actions={polish:2800,rework:8800,shorten:14800,idea:22000,undo:31500,redo:34500,copy:37000};
    for (const [name,at] of Object.entries(actions)) buttons[name].classList.toggle('active',t>=at&&t<at+700);
    if (!document.hidden) frame=requestAnimationFrame(draw);
  }
  function reset(){cancelAnimationFrame(frame);start=performance.now();draw(start)}
  document.addEventListener('visibilitychange',reset);reduced.addEventListener('change',reset);reset();
})();
