import { describe, expect, it, vi } from "vitest";
import { polishText, polishTweet } from "./polish-api";

describe("polishTweet", () => {
  it("requests a single 280-character post while preserving input", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(
        JSON.stringify({
          status: "completed",
          output: [{ content: [{ type: "output_text", text: "A post." }] }],
        }),
        { status: 200 },
      ),
    );
    await expect(
      polishTweet("Original text", "test-key", fetcher),
    ).resolves.toBe("A post.");
    const body = JSON.parse(String(fetcher.mock.calls[0][1]?.body));
    expect(body.input).toBe("Original text");
    expect(body.instructions).toContain("at most 280 characters");
    expect(body.instructions).toContain("Do not invent facts");
    expect(body.store).toBe(false);
  });
});

describe("polishText", () => {
  it("sends the selected text to OpenAI and returns only the completed output", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(
        JSON.stringify({
          status: "completed",
          output: [
            { content: [{ type: "output_text", text: "This is correct." }] },
          ],
        }),
        { status: 200 },
      ),
    );
    await expect(
      polishText("This is corect.", "test-key", fetcher),
    ).resolves.toBe("This is correct.");
    const [url, options] = fetcher.mock.calls[0];
    expect(url).toBe("https://api.openai.com/v1/responses");
    expect(JSON.parse(String(options?.body))).toMatchObject({
      model: "gpt-6-luna",
      input: "This is corect.",
      store: false,
    });
  });
  it("does not return partial or missing output", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(
        JSON.stringify({
          status: "incomplete",
          output: [{ content: [{ type: "output_text", text: "partial" }] }],
        }),
        { status: 200 },
      ),
    );
    await expect(polishText("text", "test-key", fetcher)).rejects.toThrow(
      "did not complete",
    );
  });
  it("reports authentication errors without exposing the key", async () => {
    const fetcher = vi
      .fn<typeof fetch>()
      .mockResolvedValue(new Response("secret detail", { status: 401 }));
    await expect(polishText("text", "test-secret", fetcher)).rejects.toThrow(
      "Check your OpenAI API key",
    );
  });
});
