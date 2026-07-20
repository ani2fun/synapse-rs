import { describe, expect, it } from "vitest";
import { titleForLesson } from "./seo";

describe("seo", () => {
  // Must match platform::static_routes::title_for, or the tab changes on navigation.
  it("the book leads the lesson title", () => {
    expect(titleForLesson("DSA", "Singly linked lists")).toBe("DSA · Singly linked lists — Synapse");
  });

  // The index may not have loaded yet; a lesson title alone still beats the placeholder.
  it("an unknown book still produces a usable title", () => {
    expect(titleForLesson(null, "Intro")).toBe("Intro — Synapse");
  });
});
