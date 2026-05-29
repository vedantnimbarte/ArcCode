"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { adjacentDocs } from "@/lib/nav";

export function DocsPager() {
  const pathname = usePathname();
  const { prev, next } = adjacentDocs(pathname);

  if (!prev && !next) return null;

  return (
    <nav className="mt-16 grid gap-4 border-t border-[var(--border)] pt-8 sm:grid-cols-2">
      {prev ? (
        <Link
          href={prev.href}
          className="group rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4 transition-colors hover:border-[var(--border-strong)]"
        >
          <span className="text-xs text-[var(--text-dim)]">Previous</span>
          <span className="mt-1 flex items-center gap-1.5 font-semibold text-[var(--text-strong)]">
            <span className="transition-transform group-hover:-translate-x-0.5">
              ←
            </span>
            {prev.title}
          </span>
        </Link>
      ) : (
        <span />
      )}
      {next ? (
        <Link
          href={next.href}
          className="group rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4 text-right transition-colors hover:border-[var(--border-strong)] sm:col-start-2"
        >
          <span className="text-xs text-[var(--text-dim)]">Next</span>
          <span className="mt-1 flex items-center justify-end gap-1.5 font-semibold text-[var(--text-strong)]">
            {next.title}
            <span className="transition-transform group-hover:translate-x-0.5">
              →
            </span>
          </span>
        </Link>
      ) : (
        <span />
      )}
    </nav>
  );
}
