import { polishTweet } from "./polish-api";
import { runPolish } from "./polish";

export default async function Command() {
  await runPolish(polishTweet, "tweet");
}
