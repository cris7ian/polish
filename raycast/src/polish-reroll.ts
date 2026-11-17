import { rerollText } from "./polish-api";
import { runPolish } from "./polish";

export default async function Command() {
  await runPolish(rerollText, "reroll");
}
