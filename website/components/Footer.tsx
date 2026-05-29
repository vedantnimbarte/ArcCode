import Link from "next/link";
import { Logo } from "./Logo";
import { GitHubIcon } from "./icons";
import { site } from "@/lib/site";

const cols: { title: string; links: { label: string; href: string }[] }[] = [
  {
    title: "Product",
    links: [
      { label: "Features", href: "/#features" },
      { label: "Providers", href: "/#providers" },
      { label: "Pilot mode", href: "/#pilot" },
      { label: "Learning loop", href: "/#learning" },
    ],
  },
  {
    title: "Docs",
    links: [
      { label: "Installation", href: "/docs/installation" },
      { label: "Quick start", href: "/docs/quick-start" },
      { label: "Configuration", href: "/docs/configuration" },
      { label: "CLI reference", href: "/docs/cli-reference" },
    ],
  },
  {
    title: "Reference",
    links: [
      { label: "Built-in tools", href: "/docs/tools" },
      { label: "Self-improving loop", href: "/docs/learning-loop" },
      { label: "Architecture", href: "/docs/architecture" },
      { label: "Roadmap", href: "/docs/roadmap" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="relative z-10 border-t border-[var(--border)] bg-[var(--color-carbon-200)]">
      <div className="container-page py-14">
        <div className="grid gap-10 md:grid-cols-[1.4fr_repeat(3,1fr)]">
          <div className="max-w-xs">
            <Logo />
            <p className="mt-4 text-sm leading-6 text-[var(--text-dim)]">
              {site.tagline} Provider-agnostic, local-first, written in Rust.
            </p>
            <Link
              href={site.github}
              target="_blank"
              rel="noreferrer"
              className="mt-5 inline-flex items-center gap-2 text-sm font-medium text-[var(--text-muted)] transition-colors hover:text-[var(--text-strong)]"
            >
              <GitHubIcon className="h-4 w-4" />
              {site.repoShort}
            </Link>
          </div>

          {cols.map((col) => (
            <div key={col.title}>
              <h4 className="text-xs font-bold uppercase tracking-wider text-[var(--text-dim)]">
                {col.title}
              </h4>
              <ul className="mt-4 space-y-3">
                {col.links.map((l) => (
                  <li key={l.href}>
                    <Link
                      href={l.href}
                      className="text-sm text-[var(--text-muted)] transition-colors hover:text-[var(--text-strong)]"
                    >
                      {l.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="mt-12 flex flex-col items-start justify-between gap-3 border-t border-[var(--border)] pt-6 text-sm text-[var(--text-dim)] sm:flex-row sm:items-center">
          <p>
            © {site.name}. Dual-licensed under {site.license}.
          </p>
          <p className="font-mono text-xs">
            an open alternative to Claude Code · Cursor · Aider
          </p>
        </div>
      </div>
    </footer>
  );
}
