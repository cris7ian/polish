(() => {
  const button = document.querySelector('.copy-command');
  const command = document.querySelector('#brew-command');
  const status = document.querySelector('#brew-copy-status');
  const container = document.querySelector('.brew-install');
  if (!button || !command || !status || !container) return;

  let flashTimer;
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(command.textContent);
      status.textContent = 'Homebrew install command copied.';
      status.classList.remove('copy-error');
      container.classList.remove('copy-success');
      void container.offsetWidth;
      container.classList.add('copy-success');
      clearTimeout(flashTimer);
      flashTimer = setTimeout(() => container.classList.remove('copy-success'), 1600);
    } catch {
      container.classList.remove('copy-success');
      status.textContent = 'Could not copy. Select the command to copy it.';
      status.classList.add('copy-error');
    }
  });
})();
