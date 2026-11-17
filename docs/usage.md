# Using Polish

## Editor

Select text before choosing an action. With no selection, Polish uses the whole document. **Continue idea** adds one sentence after the selection. Use Undo to reverse an edit.

| Shortcut | Action |
| --- | --- |
| ⌘R | Rework |
| ⌘P | Polish |
| ⌘. | Shorten |
| ⌘T | Tweet |
| ⌘Z | Undo |
| ⌘⇧Z | Redo |

The Tweet action rejects results longer than 280 Unicode characters. X may count some characters differently.

Paste on startup, sounds, and the menu bar icon can be changed in Settings. The API key status shows whether OpenAI accepted the key; it does not confirm model access or billing. The key copy button keeps the key on the clipboard for up to 30 seconds.

## Other apps

Select text and choose **Polish** or **Reroll** from the menu bar. Grant Accessibility access in **System Settings → Privacy & Security → Accessibility** when prompted.

Polish uses the full document when the source app exposes it. Otherwise, it uses the selected text and may invoke the app's Copy command. When no text is selected, it uses the whole document only if available.

Menu actions replace the clipboard with the result and attempt a native paste. If the source changes, Polish skips the paste and leaves the result on the clipboard. Apps differ in their support for selection checks, Undo, and reselecting the replacement.

The macOS Services menu also offers Polish and Polish Tweet. If they are missing, enable them in **System Settings → Keyboard → Keyboard Shortcuts → Services → Text**. Reopen the source app if its menu does not refresh. Services may remove rich formatting.

## Updates

Choose **Check for Updates** in the menu bar. Before installing an update, copy any editor text you want to keep and close the editor. Wait for menu actions to finish.

Versions before 1.2.0 do not include update checks. Install a newer release manually or with Homebrew.
