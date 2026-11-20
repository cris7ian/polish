# Polish

Polish is a macOS writing app for correcting, rewriting, and shortening text while keeping your voice.

## Install

Download the [latest release](https://github.com/cris7ian/polish/releases/latest) or install with Homebrew:

```sh
brew install --cask cris7ian/tap/polish
```

The release supports Apple silicon Macs running macOS 13 or later. Open Polish, sign in with **Continue with ChatGPT** to use your ChatGPT Plus or Pro plan, or add an OpenAI API key in Settings, and choose a model.

## Use

Write or paste text, select a passage, and choose **Polish**, **Rework**, **Shorten**, **Tweet**, or **Continue idea**. Without a selection, the action uses the whole document.

Use the menu bar for Polish and Reroll in other apps. The first use requires Accessibility permission. The menu copies the result and attempts to paste it; paste manually if needed.

The menu bar also has **Check for Updates**. Polish can check for updates automatically.

## Privacy

Polish sends text to OpenAI only when you choose an action. Editor actions send the whole document and selected passage. Menu actions send the available document context, or only the selection when the app does not expose the document. API keys and sign-in credentials stay in macOS Keychain. OpenAI API usage is billed separately; ChatGPT plan usage draws on the allowance your plan already includes, and you can cap it in ChatGPT Settings → Usage.

The editor pastes clipboard text on startup by default; you can turn this off in Settings.

## More

- [Using Polish](docs/usage.md)
- [Build, test, and release](docs/development.md)
- [Raycast extension](raycast/README.md)
- [Website](https://polish.salsaparapizza.com/)
