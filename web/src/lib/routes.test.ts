import { describe, expect, it } from "vitest";
import { pageFromSegments, pageUrl, segmentsOf } from "./routes";

describe("routes", () => {
  it("parses the app map", () => {
    expect(pageFromSegments([])).toEqual({ kind: "library" });
    expect(pageFromSegments(["", ""])).toEqual({ kind: "library" });
    expect(pageFromSegments(["synapse", "learn", "dsa", "intro"])).toEqual({
      kind: "lesson",
      path: ["learn", "dsa", "intro"],
    });
    // A bare /synapse is not a lesson.
    expect(pageFromSegments(["synapse"])).toEqual({ kind: "notFound", raw: "synapse" });
    expect(pageFromSegments(["blog"])).toEqual({ kind: "blog" });
    expect(pageFromSegments(["account"])).toEqual({ kind: "account" });
    expect(pageFromSegments(["admin"])).toEqual({ kind: "admin" });
    expect(pageFromSegments(["blog", "hello"])).toEqual({ kind: "blogPost", slug: "hello" });
    // Blog posts are flat — deeper paths are not pages.
    expect(pageFromSegments(["blog", "a", "b"])).toEqual({ kind: "notFound", raw: "blog/a/b" });
    expect(pageFromSegments(["ghost", "town"])).toEqual({ kind: "notFound", raw: "ghost/town" });
  });

  it("urls round trip", () => {
    expect(pageUrl({ kind: "lesson", path: ["learn", "dsa", "intro"] })).toBe(
      "/synapse/learn/dsa/intro",
    );
    expect(pageUrl({ kind: "library" })).toBe("/");
    expect(pageUrl({ kind: "blog" })).toBe("/blog");
    expect(pageUrl({ kind: "blogPost", slug: "hello" })).toBe("/blog/hello");
    expect(segmentsOf("a//b/c/")).toEqual(["a", "b", "c"]);
  });
});
