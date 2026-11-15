import {
  Clipboard,
  getFrontmostApplication,
  getPreferenceValues,
  getSelectedText,
  showHUD,
  showToast,
  Toast,
} from "@raycast/api";
import { polishText } from "./polish-api";

interface Preferences {
  openaiApiKey: string;
}

export async function runPolish(
  transform: (input: string, apiKey: string) => Promise<string>,
  kind: "text" | "tweet",
) {
  try {
    const { openaiApiKey } = getPreferenceValues<Preferences>();
    if (!openaiApiKey?.trim())
      throw new Error("Add your OpenAI API key in Raycast preferences.");
    const originalApp = await getFrontmostApplication();
    let selected: string | undefined;
    try {
      const text = await getSelectedText();
      if (text.trim()) selected = text;
    } catch {
      /* The active app may not expose a selection. */
    }
    const input = selected ?? (await Clipboard.readText());
    if (!input?.trim())
      throw new Error("Select text or copy text to the clipboard first.");
    await showToast({
      style: Toast.Style.Animated,
      title: kind === "tweet" ? "Polishing tweet…" : "Polishing text…",
    });
    const result = await transform(input, openaiApiKey.trim());
    if (kind === "tweet" && Array.from(result).length > 280)
      throw new Error("The tweet exceeds 280 characters. Try again.");
    const currentApp = await getFrontmostApplication();
    let selectionUnchanged = !selected;
    if (selected && currentApp.path === originalApp.path) {
      try {
        selectionUnchanged = (await getSelectedText()) === selected;
      } catch {
        selectionUnchanged = false;
      }
    }
    if (currentApp.path !== originalApp.path || !selectionUnchanged) {
      await Clipboard.copy(result);
      await showToast({
        style: Toast.Style.Success,
        title: "Polished text copied",
        message: "The original selection changed; paste it where you want.",
      });
      return;
    }
    await Clipboard.paste(result);
    await showHUD("Polished and pasted");
  } catch (error) {
    await showToast({
      style: Toast.Style.Failure,
      title: "Could not polish text",
      message: error instanceof Error ? error.message : "Please try again.",
    });
  }
}

export default async function Command() {
  await runPolish(polishText, "text");
}
