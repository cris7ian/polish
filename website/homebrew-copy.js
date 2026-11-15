(() => {
  const button = document.querySelector('.copy-command');
  const command = document.querySelector('#brew-command');
  const status = document.querySelector('#brew-copy-status');
  if (!button || !command || !status) return;

  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(command.textContent);
      status.textContent = 'Homebrew install command copied.';
    } catch {
      status.textContent = 'Could not copy. Select the command to copy it.';
    }
  });
})();
