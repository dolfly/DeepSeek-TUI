import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

function pageSource(path: string): string {
  return readFileSync(new URL(`../app/[locale]/${path}`, import.meta.url), "utf8");
}

describe("public website copy contracts", () => {
  it("keeps the docs hub on the compact ocean portal instead of the old almanac treatment", () => {
    const layout = pageSource("docs/layout.tsx");
    const search = readFileSync(new URL("../components/docs-search.tsx", import.meta.url), "utf8");

    expect(layout).toContain("docs-portal-hero");
    expect(layout).toContain("Find the guidance you need.");
    expect(layout).not.toContain("Section 02");
    expect(layout).not.toContain("How Codewhale works: ego");
    expect(layout).not.toContain("<Seal");
    expect(search).toContain("docs-topic-row");
    expect(search).not.toContain("40+ Markdown documents");
  });

  it("does not rule out the managed app or make it a requirement for local use", () => {
    const roadmap = pageSource("roadmap/page.tsx");

    expect(roadmap).toContain("Managed app preview and optional accounts");
    expect(roadmap).toContain("Required account for the local runtime");
    expect(roadmap).not.toContain("Hosted SaaS dashboard");
    expect(roadmap).not.toContain("Required login / accounts");
  });

  it("describes ACP and the VS Code extension at their implemented capability level", () => {
    const runtime = pageSource("runtime/page.tsx");

    expect(runtime).toContain("ACP (Agent Client Protocol)");
    expect(runtime).toContain("Baseline JSON-RPC adapter over stdio");
    expect(runtime).toContain("Phase 0 companion for the local runtime");
    expect(runtime).not.toContain("Agent Communication Protocol");
    expect(runtime).not.toContain("IETF-standard");
    expect(runtime).not.toContain("embeds Codewhale as a side-panel agent");
  });

  it("presents providers as peers and puts contributor actions near the top", () => {
    const providerCopy = `${pageSource("models/page.tsx")}\n${pageSource("faq/page.tsx")}`;
    const community = pageSource("community/page.tsx");

    expect(providerCopy).not.toMatch(/first-class|一级支持|一级模型/);
    expect(community).toContain("International open-source community");
    expect(community).toContain("issues/new/choose");
    expect(community).toContain("docs/LOCALIZATION.md");
    expect(community).toContain("Hmbown/CodeWhale/pulls");
  });
});
