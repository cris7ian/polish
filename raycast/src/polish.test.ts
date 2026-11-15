import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  selected: vi.fn(),
  app: vi.fn(),
  clipboard: vi.fn(),
  copy: vi.fn(),
  paste: vi.fn(),
  toast: vi.fn(),
  hud: vi.fn(),
  polish: vi.fn(),
  tweet: vi.fn(),
}));
vi.mock("@raycast/api", () => ({
  Clipboard: {
    readText: mocks.clipboard,
    copy: mocks.copy,
    paste: mocks.paste,
  },
  getFrontmostApplication: mocks.app,
  getSelectedText: mocks.selected,
  getPreferenceValues: () => ({ openaiApiKey: "test-key" }),
  showToast: mocks.toast,
  showHUD: mocks.hud,
  Toast: {
    Style: { Animated: "animated", Success: "success", Failure: "failure" },
  },
}));
vi.mock("./polish-api", () => ({
  polishText: mocks.polish,
  polishTweet: mocks.tweet,
}));
import Command from "./polish";
import TweetCommand from "./polish-tweet";

describe("Polish command", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.app.mockResolvedValue({ path: "/Applications/TextEdit.app" });
    mocks.selected.mockResolvedValue("Bad speling");
    mocks.polish.mockResolvedValue("Bad spelling");
  });
  it("uses the tweet action and copies when the selection changes", async () => {
    mocks.tweet.mockResolvedValue("A concise post");
    mocks.selected
      .mockResolvedValueOnce("Bad speling")
      .mockResolvedValueOnce("Changed");
    await TweetCommand();
    expect(mocks.tweet).toHaveBeenCalledWith("Bad speling", "test-key");
    expect(mocks.paste).not.toHaveBeenCalled();
    expect(mocks.copy).toHaveBeenCalledWith("A concise post");
  });
  it("accepts 280 Unicode scalar values and rejects 281 without changing text or clipboard", async () => {
    mocks.tweet.mockResolvedValue("😀".repeat(280));
    await TweetCommand();
    expect(mocks.paste).toHaveBeenCalledWith("😀".repeat(280));
    mocks.paste.mockClear();
    mocks.tweet.mockResolvedValue("😀".repeat(281));
    await TweetCommand();
    expect(mocks.paste).not.toHaveBeenCalled();
    expect(mocks.copy).not.toHaveBeenCalled();
    expect(mocks.toast).toHaveBeenLastCalledWith(
      expect.objectContaining({
        style: "failure",
        message: "The tweet exceeds 280 characters. Try again.",
      }),
    );
  });
  it("pastes when the original selection remains active", async () => {
    await Command();
    expect(mocks.polish).toHaveBeenCalledWith("Bad speling", "test-key");
    expect(mocks.paste).toHaveBeenCalledWith("Bad spelling");
  });
  it("copies instead of pasting when the selection changes during the request", async () => {
    mocks.selected
      .mockResolvedValueOnce("Bad speling")
      .mockResolvedValueOnce("Something else");
    await Command();
    expect(mocks.paste).not.toHaveBeenCalled();
    expect(mocks.copy).toHaveBeenCalledWith("Bad spelling");
  });
  it("copies instead of pasting when the active app changes", async () => {
    mocks.app
      .mockResolvedValueOnce({ path: "/Applications/TextEdit.app" })
      .mockResolvedValueOnce({ path: "/Applications/Other.app" });
    await Command();
    expect(mocks.paste).not.toHaveBeenCalled();
    expect(mocks.copy).toHaveBeenCalledWith("Bad spelling");
  });
  it("leaves text unchanged on API failure", async () => {
    mocks.polish.mockRejectedValue(new Error("Request failed"));
    await Command();
    expect(mocks.paste).not.toHaveBeenCalled();
    expect(mocks.copy).not.toHaveBeenCalled();
    expect(mocks.toast).toHaveBeenCalledWith(
      expect.objectContaining({ style: "failure" }),
    );
  });
  it("uses the clipboard when selected text is unavailable", async () => {
    mocks.selected.mockRejectedValue(new Error("No selection"));
    mocks.clipboard.mockResolvedValue("Copied text");
    await Command();
    expect(mocks.polish).toHaveBeenCalledWith("Copied text", "test-key");
    expect(mocks.paste).toHaveBeenCalledWith("Bad spelling");
  });
});
