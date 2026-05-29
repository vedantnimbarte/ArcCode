"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Logo } from "./Logo";
import { primaryNav, site } from "@/lib/site";
import { GitHubIcon } from "./icons";

export function Nav() {
  const [scrolled, setScrolled] = useState(false);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 12);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  useEffect(() => {
    document.body.style.overflow = open ? "hidden" : "";
    return () => {
      document.body.style.overflow = "";
    };
  }, [open]);

  return (
    <header
      className={`fixed inset-x-0 top-0 z-50 transition-all duration-300 ${
        scrolled
          ? "border-b border-[var(--border)] bg-[var(--bg)]/80 backdrop-blur-xl"
          : "border-b border-transparent"
      }`}
    >
      <nav className="container-page flex h-16 items-center justify-between gap-4">
        <Logo />

        <div className="hidden items-center gap-1 md:flex">
          {primaryNav.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className="rounded-md px-3 py-2 text-sm font-medium text-[var(--text-muted)] transition-colors hover:text-[var(--text-strong)]"
            >
              {item.label}
            </Link>
          ))}
        </div>

        <div className="flex items-center gap-2">
          <Link
            href={site.github}
            target="_blank"
            rel="noreferrer"
            className="hidden items-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--color-carbon-300)] px-3.5 py-2 text-sm font-semibold text-[var(--text)] transition-colors hover:border-[var(--border-strong)] hover:text-[var(--text-strong)] sm:inline-flex"
          >
            <GitHubIcon className="h-4 w-4" />
            GitHub
          </Link>
          <Link
            href="/docs/installation"
            className="hidden rounded-lg bg-[var(--color-snow)] px-3.5 py-2 text-sm font-bold text-[var(--color-carbon-100)] transition-transform hover:-translate-y-0.5 sm:inline-flex"
          >
            Get started
          </Link>

          <button
            type="button"
            aria-label="Toggle menu"
            aria-expanded={open}
            onClick={() => setOpen((v) => !v)}
            className="grid h-10 w-10 place-items-center rounded-lg border border-[var(--border)] text-[var(--text)] md:hidden"
          >
            <span className="relative block h-4 w-5">
              <span
                className={`absolute left-0 block h-0.5 w-5 bg-current transition-all ${open ? "top-2 rotate-45" : "top-0.5"}`}
              />
              <span
                className={`absolute left-0 top-2 block h-0.5 w-5 bg-current transition-all ${open ? "opacity-0" : "opacity-100"}`}
              />
              <span
                className={`absolute left-0 block h-0.5 w-5 bg-current transition-all ${open ? "top-2 -rotate-45" : "top-[14px]"}`}
              />
            </span>
          </button>
        </div>
      </nav>

      <AnimatePresence>
        {open && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
            className="overflow-hidden border-b border-[var(--border)] bg-[var(--bg)]/95 backdrop-blur-xl md:hidden"
          >
            <div className="container-page flex flex-col gap-1 py-4">
              {primaryNav.map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  onClick={() => setOpen(false)}
                  className="rounded-lg px-3 py-3 text-base font-medium text-[var(--text-muted)] transition-colors hover:bg-[var(--color-carbon-300)] hover:text-[var(--text-strong)]"
                >
                  {item.label}
                </Link>
              ))}
              <div className="mt-2 flex gap-2">
                <Link
                  href={site.github}
                  target="_blank"
                  rel="noreferrer"
                  className="inline-flex flex-1 items-center justify-center gap-2 rounded-lg border border-[var(--border)] px-3 py-2.5 text-sm font-semibold text-[var(--text)]"
                >
                  <GitHubIcon className="h-4 w-4" /> GitHub
                </Link>
                <Link
                  href="/docs/installation"
                  onClick={() => setOpen(false)}
                  className="inline-flex flex-1 items-center justify-center rounded-lg bg-[var(--color-snow)] px-3 py-2.5 text-sm font-bold text-[var(--color-carbon-100)]"
                >
                  Get started
                </Link>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </header>
  );
}
