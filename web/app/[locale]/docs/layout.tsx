import Link from "next/link";
import { Whale } from "@/components/whale";
import { getTopicsByCategory, REPO_DOCS_BASE, type DocTopic } from "@/lib/docs-map";

/* ------------------------------------------------------------------ */
/*  Locale-aware category heading labels                               */
/* ------------------------------------------------------------------ */

const CATEGORY_LABELS: Record<string, { en: string; zh: string }> = {
  "getting-started": { en: "Getting started", zh: "入门" },
  "core-concepts": { en: "Core concepts", zh: "核心概念" },
  reference: { en: "Reference", zh: "参考" },
  extending: { en: "Extending", zh: "扩展" },
  operations: { en: "Operations", zh: "运维" },
};

/* ------------------------------------------------------------------ */
/*  Link resolution helpers                                            */
/* ------------------------------------------------------------------ */

function topicHref(topic: DocTopic, locale: string): string {
  if (topic.hasPage) {
    return `/${locale}/docs/${topic.slug}`;
  }
  const src = Array.isArray(topic.repoSource) ? topic.repoSource[0] : topic.repoSource;
  return `${REPO_DOCS_BASE}/${src}`;
}

/* ------------------------------------------------------------------ */
/*  Sidebar                                                            */
/* ------------------------------------------------------------------ */

function DocsSidebar({ locale, currentId }: { locale: string; currentId?: string }) {
  const isZh = locale === "zh";
  const byCategory = getTopicsByCategory();

  return (
    <aside className="docs-sidebar min-w-0">
      <div className="lg:sticky lg:top-24">
        <div className="docs-sidebar-heading">{isZh ? "文档目录" : "Documentation"}</div>
        <nav aria-label={isZh ? "文档目录" : "Documentation index"}>
          {[...byCategory.entries()].map(([cat, topics]) => (
            <div key={cat} className="docs-sidebar-group">
              <div className="docs-sidebar-category">
                {isZh ? CATEGORY_LABELS[cat]?.zh ?? cat : CATEGORY_LABELS[cat]?.en ?? cat}
              </div>
              <ul>
                {topics.map((t) => {
                  const href = topicHref(t, locale);
                  const isCurrent = t.id === currentId;
                  return (
                    <li key={t.id}>
                      <Link
                        href={href}
                        target={t.hasPage ? undefined : "_blank"}
                        rel={t.hasPage ? undefined : "noreferrer"}
                        aria-current={isCurrent ? "page" : undefined}
                        className={isCurrent ? "docs-sidebar-link docs-sidebar-link-current" : "docs-sidebar-link"}
                      >
                        <span>{isZh ? t.label.zh : t.label.en}</span>
                        {!t.hasPage && <span aria-hidden="true">↗</span>}
                      </Link>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </nav>
      </div>
    </aside>
  );
}

/* ------------------------------------------------------------------ */
/*  Layout (Next.js App Router)                                        */
/* ------------------------------------------------------------------ */

export default async function DocsLayout({
  children,
  params,
}: {
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  const isZh = locale === "zh";

  return (
    <div className="docs-theme docs-portal min-h-screen">
      <section className="docs-portal-hero">
        <div className="portal-current" aria-hidden="true" />
        <div className="portal-container docs-portal-hero-inner">
          <div className="portal-mark">
            <Whale size={28} className="text-current" />
            <span>{isZh ? "Codewhale 文档" : "Codewhale documentation"}</span>
          </div>
          <h1>{isZh ? "查找准确的使用说明。" : "Find the guidance you need."}</h1>
          <p>
            {isZh
              ? "从安装和首次运行开始，或者直接查找模式、权限、工具、提供商、Fleet、MCP 与运行时 API。网站页面提供简明入口，仓库中的源文档保留完整细节。"
              : "Start with installation and first run, or go straight to modes, permissions, tools, providers, Fleet, MCP, and the Runtime API. These pages provide a clear index while the source documents in the repository carry the full detail."}
          </p>
          <div className="portal-actions">
            <Link href={`/${locale}/install`} className="portal-button portal-button-primary">
              {isZh ? "安装 Codewhale" : "Install Codewhale"}
            </Link>
            <Link
              href="https://github.com/Hmbown/CodeWhale/tree/main/docs"
              target="_blank"
              rel="noreferrer"
              className="portal-button portal-button-secondary"
            >
              {isZh ? "浏览源文档 ↗" : "Browse source docs ↗"}
            </Link>
          </div>
        </div>
      </section>

      <div className="portal-container docs-shell min-w-0">
        <DocsSidebar locale={locale} />
        <article className="docs-content min-w-0">{children}</article>
      </div>
    </div>
  );
}
