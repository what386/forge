import fs from "node:fs/promises";
import path from "node:path";
import MarkdownIt from "markdown-it";

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname));
const DOCS_ROOT = path.resolve(ROOT, "..");
const LUA_DIR = path.join(DOCS_ROOT, "lua");
const SITE_DIR = path.join(ROOT, "site");
const ASSETS_DIR = path.join(SITE_DIR, "assets");

const SITE_TITLE = "Forge Lua API";
const STORAGE_KEY = "forge-lua-docs-theme";

const pages = [
  { title: "Overview", file: "index.md", url: "index.html", section: "Start" },
  { title: "Getting Started", file: "getting-started.md", url: "getting-started.html", section: "Start" },
  { title: "Arguments", file: "arguments.md", url: "arguments.html", section: "Authoring" },
  { title: "Rendering", file: "rendering.md", url: "rendering.html", section: "Authoring" },
  { title: "Filesystem", file: "filesystem.md", url: "filesystem.html", section: "Runtime APIs" },
  { title: "Commands", file: "commands.md", url: "commands.html", section: "Runtime APIs" },
  { title: "Prompts", file: "prompts.md", url: "prompts.html", section: "Runtime APIs" },
  { title: "Standard Library", file: "stdlib.md", url: "stdlib.html", section: "Runtime APIs" },
  { title: "Hooks and Control", file: "hooks.md", url: "hooks.html", section: "Runtime APIs" },
  { title: "Security", file: "security.md", url: "security.html", section: "Runtime APIs" },
  { title: "API Reference", file: "reference.md", url: "reference.html", section: "Reference" },
  { title: "Manifest Reference", file: "../manifests.md", url: "manifest.html", section: "Reference" },
];

const md = new MarkdownIt({
  html: false,
  linkify: true,
  typographer: true,
});

function slugify(value) {
  return value
    .toLowerCase()
    .replace(/`/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

md.renderer.rules.heading_open = (tokens, idx, options, env, self) => {
  const next = tokens[idx + 1];
  if (next?.type === "inline") {
    tokens[idx].attrSet("id", slugify(next.content));
  }
  return self.renderToken(tokens, idx, options);
};

function escapeHtml(value) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function stripHtml(html) {
  return html.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
}

function collectHeadings(markdown) {
  const headings = [];
  const re = /^(#{2,3})\s+(.+)$/gm;
  let match;
  while ((match = re.exec(markdown)) !== null) {
    headings.push({
      depth: match[1].length,
      text: match[2].trim(),
    });
  }
  return headings;
}

function pageMap() {
  return new Map(pages.map((page) => [page.file, page.url]));
}

function rewriteMarkdownLinks(markdown, currentFile, fileToUrl) {
  return markdown.replace(/\]\(([^)]+)\)/g, (full, rawHref) => {
    const [href, hash = ""] = rawHref.split("#");
    if (/^(https?:|mailto:)/.test(href) || href === "") return full;

    const normalized = path.posix.normalize(path.posix.join(path.posix.dirname(currentFile), href));
    const luaRelative = normalized.replace(/^lua\//, "");
    const target = fileToUrl.get(luaRelative);
    if (target) {
      return `](${target}${hash ? `#${hash}` : ""})`;
    }

    return full;
  });
}

function groupNav(currentUrl) {
  const sections = [];
  for (const page of pages) {
    let section = sections.find((item) => item.name === page.section);
    if (!section) {
      section = { name: page.section, pages: [] };
      sections.push(section);
    }
    section.pages.push(page);
  }

  return sections
    .map((section) => {
      const links = section.pages
        .map((page) => {
          const active = page.url === currentUrl ? ' class="active" aria-current="page"' : "";
          return `<li><a${active} href="${page.url}">${escapeHtml(page.title)}</a></li>`;
        })
        .join("");
      return `<section class="nav-section"><h2>${escapeHtml(section.name)}</h2><ul>${links}</ul></section>`;
    })
    .join("");
}

function renderToc(headings) {
  if (headings.length === 0) return "";
  const links = headings
    .filter((heading) => heading.depth === 2)
    .map((heading) => {
      const slug = slugify(heading.text);
      return `<a href="#${escapeHtml(slug)}">${escapeHtml(heading.text)}</a>`;
    })
    .join("");
  return `<aside class="toc" aria-label="Page contents"><h2>On This Page</h2>${links}</aside>`;
}

function renderPage({ page, contentHtml, headings, prev, next }) {
  const prevHtml = prev
    ? `<a class="pager-link" href="${prev.url}"><span>Previous</span>${escapeHtml(prev.title)}</a>`
    : '<span class="pager-spacer"></span>';
  const nextHtml = next
    ? `<a class="pager-link next" href="${next.url}"><span>Next</span>${escapeHtml(next.title)}</a>`
    : '<span class="pager-spacer"></span>';

  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${escapeHtml(page.title)} | ${escapeHtml(SITE_TITLE)}</title>
  <script>
    (() => {
      const key = ${JSON.stringify(STORAGE_KEY)};
      const stored = (() => {
        try { return localStorage.getItem(key); } catch { return null; }
      })();
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      document.documentElement.dataset.theme = stored || (prefersDark ? "dark" : "light");
      document.documentElement.dataset.themeKey = key;
    })();
  </script>
  <link rel="stylesheet" href="assets/styles.css">
</head>
<body>
  <header class="topbar">
    <button class="nav-toggle" id="nav-toggle" type="button" aria-label="Toggle navigation">Menu</button>
    <a class="brand" href="index.html">
      <span class="brand-mark">F</span>
      <span>${escapeHtml(SITE_TITLE)}</span>
    </a>
    <label class="search-wrap" for="search-input">
      <span class="search-label">Search</span>
      <input id="search-input" type="search" placeholder="Find APIs, guides, examples" autocomplete="off">
    </label>
    <button class="theme-toggle" id="theme-toggle" type="button" aria-label="Switch color theme" aria-pressed="false">
      <span class="theme-toggle-icon" aria-hidden="true"></span>
      <span class="theme-toggle-text">Theme</span>
    </button>
  </header>

  <div class="layout">
    <aside class="sidebar" id="sidebar">
      <nav aria-label="Documentation">
        ${groupNav(page.url)}
      </nav>
      <div class="search-results" id="search-results" aria-live="polite"></div>
    </aside>

    <main class="page-shell" id="main-content">
      <div class="content">
        <article>${contentHtml}</article>
        <footer class="pager">${prevHtml}${nextHtml}</footer>
      </div>
      ${renderToc(headings)}
    </main>
  </div>

  <script src="assets/app.js" defer></script>
</body>
</html>`;
}

async function main() {
  const fileToUrl = pageMap();

  await fs.rm(SITE_DIR, { recursive: true, force: true });
  await fs.mkdir(ASSETS_DIR, { recursive: true });

  const searchIndex = [];

  for (let i = 0; i < pages.length; i++) {
    const page = pages[i];
    const markdownPath = path.join(LUA_DIR, page.file);
    const markdown = await fs.readFile(markdownPath, "utf8");
    const rewritten = rewriteMarkdownLinks(markdown, page.file, fileToUrl);
    const contentHtml = md.render(rewritten);
    const headings = collectHeadings(markdown);
    const prev = i > 0 ? pages[i - 1] : null;
    const next = i < pages.length - 1 ? pages[i + 1] : null;

    await fs.writeFile(
      path.join(SITE_DIR, page.url),
      renderPage({ page, contentHtml, headings, prev, next }),
      "utf8",
    );

    searchIndex.push({
      title: page.title,
      url: page.url,
      section: page.section,
      headings: headings.map((heading) => heading.text),
      text: stripHtml(contentHtml),
    });
  }

  await fs.writeFile(path.join(SITE_DIR, "search-index.json"), JSON.stringify(searchIndex, null, 2), "utf8");
  await fs.copyFile(path.join(ROOT, "src", "styles.css"), path.join(ASSETS_DIR, "styles.css"));
  await fs.copyFile(path.join(ROOT, "src", "app.js"), path.join(ASSETS_DIR, "app.js"));

  console.log(`Generated ${pages.length} Forge Lua API pages in ${SITE_DIR}`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack || error.message : error);
  process.exit(1);
});
