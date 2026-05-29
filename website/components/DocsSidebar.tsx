"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { docsNav } from "@/lib/nav";

export function DocsSidebar() {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);

  const list = (
    <nav className="space-y-7">
      {docsNav.map((section) => (
        <div key={section.title}>
          <h4 className="mb-3 text-xs font-bold uppercase tracking-wider text-[var(--text-dim)]">
            {section.title}
          </h4>
          <ul className="space-y-0.5 border-l border-[var(--border)]">
            {section.items.map((item) => {
              const active = pathname === item.href;
              return (
                <li key={item.href}>
                  <Link
                    href={item.href}
                    onClick={() => setOpen(false)}
                    className={`-ml-px block border-l-2 py-1.5 pl-4 text-sm transition-colors ${
                      active
                        ? "border-[var(--color-french-500)] font-semibold text-[var(--text-strong)]"
                        : "border-transparent text-[var(--text-muted)] hover:border-[var(--border-strong)] hover:text-[var(--text)]"
                    }`}
                  >
                    {item.title}
                  </Link>
                </li>
              );
            })}
          </ul>
        </div>
      ))}
    </nav>
  );

  return (
    <>
      {/* mobile toggle */}
      <div className="lg:hidden">
        <button
          type="button"
          onClick={() => setOpen((v) => !v)}
          aria-expanded={open}
          className="flex w-full items-center justify-between rounded-lg border border-[var(--border)] bg-[var(--surface)] px-4 py-2.5 text-sm font-medium text-[var(--text)]"
        >
          Documentation menu
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            className={`transition-transform ${open ? "rotate-180" : ""}`}
          >
            <path d="m6 9 6 6 6-6" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </button>
        {open && (
          <div className="mt-3 rounded-lg border border-[var(--border)] bg-[var(--surface)] p-5">
            {list}
          </div>
        )}
      </div>

      {/* desktop sticky sidebar */}
      <aside className="hidden lg:block">
        <div className="sticky top-28 max-h-[calc(100vh-8rem)] overflow-y-auto pr-2">
          {list}
        </div>
      </aside>
    </>
  );
}
