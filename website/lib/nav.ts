export type DocLink = { title: string; href: string };
export type DocSection = { title: string; items: DocLink[] };

export const docsNav: DocSection[] = [
  {
    title: "Getting started",
    items: [
      { title: "Introduction", href: "/docs" },
      { title: "Installation", href: "/docs/installation" },
      { title: "Quick start", href: "/docs/quick-start" },
      { title: "Configuration", href: "/docs/configuration" },
      { title: "Permission modes", href: "/docs/permission-modes" },
    ],
  },
  {
    title: "Providers",
    items: [
      { title: "Supported providers", href: "/docs/providers" },
      { title: "Pilot provider support", href: "/docs/pilot-providers" },
    ],
  },
  {
    title: "Features",
    items: [
      { title: "CLI reference", href: "/docs/cli-reference" },
      { title: "Built-in tools", href: "/docs/tools" },
      { title: "Self-improving loop", href: "/docs/learning-loop" },
      { title: "Hooks", href: "/docs/hooks" },
      { title: "Slash commands", href: "/docs/slash-commands" },
      { title: "Pilot mode", href: "/docs/pilot" },
    ],
  },
  {
    title: "Reference",
    items: [
      { title: "Architecture", href: "/docs/architecture" },
      { title: "Roadmap", href: "/docs/roadmap" },
      { title: "Contributing", href: "/docs/contributing" },
    ],
  },
];

export const flatDocs: DocLink[] = docsNav.flatMap((s) => s.items);

export function adjacentDocs(pathname: string) {
  const i = flatDocs.findIndex((d) => d.href === pathname);
  return {
    prev: i > 0 ? flatDocs[i - 1] : null,
    next: i >= 0 && i < flatDocs.length - 1 ? flatDocs[i + 1] : null,
  };
}
