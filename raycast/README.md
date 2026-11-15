# Polish for Raycast

Polish Text corrects selected text with OpenAI and pastes the result. If no text is selected, it uses the clipboard. If the active app or selection changes during the request, it copies the result instead.

1. Run `npm install` in this directory.
2. Run `npm run dev` to import the extension into Raycast.
3. Set an OpenAI API key in the Polish Text command preferences.
4. Select or copy text and run Polish Text in Raycast.

The Raycast command stores its own API key preference. It uses GPT-6 Luna independently of the Polish app's model setting. API usage is billed separately from a ChatGPT subscription.

Run `npm run verify` to check the extension. The development watcher can stop after import; run it again to pick up changes.
