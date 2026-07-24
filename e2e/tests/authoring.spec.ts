// The content-editing surface, ANONYMOUS — so it runs on every push, no sign-in. The e2e stack
// runs the server on its `content_forge` default of "dry-run", so `/api/edits` is mounted and the
// whole feature is reachable; only a signed-in, allow-listed caller can actually propose, which
// the service_tests cover over fakes. Here we prove the PUBLIC contract: the affordance stays
// hidden from a reader, the gates answer correctly, and the pages render without a page error.
import { expect, test } from "./fixtures";

const LESSON = "/synapse/learn/smoke/intro";

test("the lesson page shows Suggest-an-edit GATED to an anonymous reader", async ({ page }) => {
  await page.goto(LESSON);
  await expect(page.locator("h1").first()).toBeVisible();
  // Visible but not actionable — the workbench Submit grammar. An anonymous reader should see the
  // affordance exists and learn from the tooltip how to ask for it, rather than meet a blank space.
  const link = page.locator("[data-edit-link]");
  await expect(link).toBeVisible();
  await expect(link).toHaveClass(/lesson-edit-link--gated/);
  await expect(link).toHaveAttribute("aria-disabled", "true");
  // The tooltip carries the request-access route (rendered by the server, so it survives no-JS).
  await expect(page.locator("[data-edit-tip]")).toHaveAttribute("data-tip", /content-editor list.*a\.r\.kakde@gmail\.com/s);
});

test("a gated Suggest-an-edit click does not navigate to the editor", async ({ page }) => {
  await page.goto(LESSON);
  const link = page.locator("[data-edit-link]");
  await expect(link).toHaveClass(/lesson-edit-link--gated/);
  // FORCED on purpose. `aria-disabled` already makes Playwright (and assistive tech) treat this as
  // disabled, so an ordinary click never lands — which is why the click has to be forced to reach
  // the thing under test: the island's own guard. `aria-disabled` is a promise, not a behaviour,
  // and a plain mouse click on an <a href> would otherwise still navigate.
  await link.click({ force: true });
  await expect(page).toHaveURL(new RegExp(`${LESSON}$`));
});

test("GET /api/edits/config reports dry-run and canEdit false for anonymous", async ({ request }) => {
  const response = await request.get("/api/edits/config");
  expect(response.status()).toBe(200);
  const config = await response.json();
  expect(config.enabled).toBe(true);
  expect(config.mode).toBe("dry-run");
  expect(config.canEdit).toBe(false);
});

test("GET /api/edits/source refuses an anonymous caller with 401", async ({ request }) => {
  const response = await request.get(`/api/edits/source/${LESSON.replace("/synapse/", "")}`);
  expect(response.status()).toBe(401);
});

test("POST /api/edits refuses an anonymous caller with 401 and commits nothing", async ({ request }) => {
  const response = await request.post("/api/edits", {
    data: { lessonPath: "learn/smoke/intro", source: "hi", baseFingerprint: "x" },
  });
  expect(response.status()).toBe(401);
});

test("GET /api/admin/content-editors refuses an anonymous caller with 401", async ({ request }) => {
  const response = await request.get("/api/admin/content-editors");
  expect(response.status()).toBe(401);
});

test("the /edit page renders its shell and the signed-out gate without a page error", async ({ page }) => {
  await page.goto(`/edit/learn/smoke/intro`);
  // The island resolves the auth store (which lands on anonymous with no session) and shows the
  // sign-in gate — not a blank page, not a thrown error (the fixtures harness fails on either).
  await expect(page.locator(".edit-gate__title")).toBeVisible({ timeout: 15_000 });
  await expect(page.getByText(/sign in/i).first()).toBeVisible();
});

test("the /admin page shows the signed-out state for an anonymous visitor", async ({ page }) => {
  await page.goto("/admin");
  await expect(page.getByText("Not signed in")).toBeVisible({ timeout: 15_000 });
  // Neither allowlist section renders its table until an admin is signed in.
  await expect(page.locator(".admin__table")).toHaveCount(0);
});
