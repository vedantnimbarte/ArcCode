"use client";

import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

type Item = { id: string; text: string; level: number };

function slugify(s: string) {
  return s
    .toLowerCase()
    .replace(/[^\w\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-");
}

export function TableOfContents() {
  const pathname = usePathname();
  const [items, setItems] = useState<Item[]>([]);
  const [active, setActive] = useState<string>("");

  useEffect(() => {
    const content = document.getElementById("docs-content");
    if (!content) return;
    const headings = Array.from(
      content.querySelectorAll("h2, h3"),
    ) as HTMLElement[];

    const next: Item[] = headings.map((h) => {
      if (!h.id) h.id = slugify(h.textContent ?? "");
      return {
        id: h.id,
        text: h.textContent ?? "",
        level: h.tagName === "H3" ? 3 : 2,
      };
    });
    setItems(next);

    const observer = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) setActive(e.target.id);
        }
      },
      { rootMargin: "-96px 0px -70% 0px", threshold: 0 },
    );
    headings.forEach((h) => observer.observe(h));
    return () => observer.disconnect();
  }, [pathname]);

  if (items.length < 2) return <div className="hidden xl:block" />;

  return (
    <aside className="hidden xl:block">
      <div className="sticky top-28">
        <h4 className="mb-3 text-xs font-bold uppercase tracking-wider text-[var(--text-dim)]">
          On this page
        </h4>
        <ul className="space-y-1.5 border-l border-[var(--border)] text-sm">
          {items.map((it) => (
            <li key={it.id}>
              <a
                href={`#${it.id}`}
                className={`-ml-px block border-l-2 py-0.5 transition-colors ${
                  it.level === 3 ? "pl-7" : "pl-4"
                } ${
                  active === it.id
                    ? "border-[var(--color-french-500)] text-[var(--text-strong)]"
                    : "border-transparent text-[var(--text-dim)] hover:text-[var(--text)]"
                }`}
              >
                {it.text}
              </a>
            </li>
          ))}
        </ul>
      </div>
    </aside>
  );
}
