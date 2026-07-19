# Step 50 — Discoverability

*(442 lessons and one title between them.)*

## What was actually wrong

`client/index.html` shipped this, and nothing else:

```html
<title>Synapse</title>
```

Hardcoded, and `document.title` was never set anywhere in `client/src` or `client/islands` —
verified by grep, not assumed. So every lesson, every book, the blog and the account page all
served the identical head. No description, no Open Graph tags, no canonical link, no
`sitemap.xml`, no `robots.txt`; `client/public/` held exactly one file, and it was the Keycloak
silent-SSO shim.

The tempting diagnosis is "it's a CSR app, so Google can't see it". That is not the problem —
Google has executed JavaScript for years, and the pages were indexable. The problem is that they
were **indistinguishable**: 442 results with the same title and no snippet, competing with each
other. And social crawlers genuinely do not run JS, so every link shared anywhere previewed as
the same grey card.

## Why not SSR

Leptos has an `ssr` feature. Turning it on means hydration, restructuring the client into
server-rendered and client-rendered halves, and giving up the CSR simplicity that the whole build
has rested on since step 02 — to buy a `<title>` tag.

The index is already served with `no-cache`, because deploys have to show. So the document is
already read per request, and rewriting a few hundred bytes of its `<head>` on the way out costs
nothing that was not already being spent. That is the whole trick: **a string substitution
against the in-memory catalog index.**

## The three things it needed

**A description to say.** `Lesson` was `{slug, title, order, essential}` — no description
anywhere in the index, and no prose-excerpt helper in the codebase. But `frontmatter.rs` already
parsed `summary:` for the per-request lesson payload, and the walker holds the raw lesson source
on the exact line it constructs a `Lesson`. So `extract_summary` mirrors `extract_essential`, and
`Lesson` gains one field.

That field is **index-only, deliberately absent from `LessonDto`**. The client already receives
`frontmatter.summary` on the lesson payload it fetches anyway; adding it to the index too would
put 442 more strings into a document every visitor downloads, to buy nothing.

**A cheap way to ask.** Neither existing service method fits a per-request head: `index()` clones
the entire catalog, and `lesson()` re-reads the file from disk and probes for an editorial
sidecar. `page_meta(path)` projects title, description and book title straight off the cached
walk, and `current_walk` stays private.

**Somewhere to put it.** `StaticRoutes` held a `PathBuf` and its handlers discarded the request
path. It now carries the catalog and an origin, `/synapse/{*rest}` routes to a handler that
resolves the lesson, and the head is rewritten before the bytes go out.

## Details that are load-bearing

**The title is replaced, not appended.** A second `<title>` element is ignored by every consumer,
so appending would have produced a page that looked correct in the response body and behaved
exactly as before. Pinned by a test asserting the original is *gone* and that exactly one
`<title>` remains.

**Everything interpolated is escaped.** Titles and summaries are authored prose. A stray `"` in a
summary is a typo; unescaped inside `content="…"` it is an injection. The test feeds a title
containing a quote, a `<script>` tag and an ampersand, and asserts none of them survive raw.

**A malformed index degrades instead of corrupting.** An unclosed `<title>` would make a naive
slice run to the end of the document; `replace_title` returns the input untouched instead.

**The origin is configured, not sniffed.** Open Graph needs absolute URLs, and `Host` is
caller-controlled — a configured `SYNAPSE_SITE_URL` cannot be poisoned by a request.

**An unknown lesson still serves the shell.** The SPA owns client-side routing and may know a
route the catalog does not, so a `page_meta` miss falls back to the site-wide head rather than
404ing the page out from under the router.

## The client half, and why the format is duplicated

The server renders the head for the URL a visitor *lands* on. That covers crawlers and shared
links, which never run JS — and it goes stale the instant the SPA navigates, because a
client-side route change does not re-fetch the document. Open lesson A, click through to B, and
the tab, the history entry and any bookmark taken along the way all still say A.

So `client/src/seo.rs` sets `document.title` and the description when a lesson loads. The title
FORMAT (`Book · Lesson — Synapse`) is duplicated from the server on purpose: the alternative is
shipping the rendered title down the wire on every payload — a new wire field and a contract to
keep in step — to save one `format!`. Both sides are pinned to the same shape by unit tests, and
the book leads because the left of a string is what survives truncation in a tab strip.

## What this deliberately does not do

**No SSR, and no prerender step.** See above.

**No `lastmod` in the sitemap.** The honest source would be the content commit date per file,
which the index does not carry. A fabricated or blanket timestamp is worse than the field's
absence — crawlers weight it.

**No per-page OG images.** `summary` cards only. A per-lesson image needs an image, and there
isn't one.

**No structured data / JSON-LD.** Nothing to say yet that the meta tags do not.

**Nothing for `/blog` posts.** The blog is a separate context with its own payload shape; it gets
the site-wide head plus its sitemap entry. Doing it properly is a blog-context change, not a
static-routes one.

## Verified

Six unit tests on the pure half — substitution, escaping, injection-into-head, and the two
degradation paths. Six integration tests through the whole router, against a temp content tree
with real frontmatter:

```
two_lessons_serve_two_different_titles ......................... ok
the_frontmatter_summary_becomes_the_description_and_falls_back .. ok
the_canonical_url_is_absolute_and_per_page ..................... ok
an_unknown_lesson_still_serves_the_spa_under_the_site_head ..... ok
the_sitemap_lists_every_lesson_absolutely ...................... ok
robots_points_at_the_sitemap_and_keeps_crawlers_off_the_api .... ok
```

The first is the regression this step exists to prevent, and it asserts the two documents
*differ* rather than merely that each contains something — the old behaviour would have passed a
weaker check.

Content and dist live in separate temp trees in those tests, because a `dist/` inside the content
root gets walked as catalog content.

423 rust (+14) + 74 vitest. Critical path 637/700 KiB gz — unchanged; `seo.rs` is three functions.

## The lesson

**"Can it be crawled" and "is it worth crawling" are different questions, and only the second
one mattered.** The reflex diagnosis for a CSR app is that the crawler cannot see it, which
points at SSR — a week of restructuring. The actual defect was that 442 pages described
themselves identically, and the fix was a string substitution against an index the server was
already holding in memory, on a document it was already re-reading on every request.
