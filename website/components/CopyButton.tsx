"use client";

import { useState } from "react";

/**
 * Copy-to-clipboard control. When no `text` is provided it copies the
 * textContent of the sibling <pre> inside its parent wrapper (used by MDX
 * code blocks).
 */
export function CopyButton({ text }: { text?: string }) {
  const [copied, setCopied] = useState(false);

  const onClick = async (e: React.MouseEvent<HTMLButtonElement>) => {
    let value = text;
    if (!value) {
      const wrapper = e.currentTarget.parentElement;
      const pre = wrapper?.querySelector("pre");
      value = pre?.textContent ?? "";
    }
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 1600);
    } catch {
      /* clipboard unavailable */
    }
  };

  return (
    <button
      type="button"
      onClick={onClick}
      aria-label="Copy code"
      className="absolute right-3 top-3 z-10 inline-flex h-8 items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--color-carbon-400)]/80 px-2.5 text-xs font-medium text-[var(--text-dim)] opacity-0 backdrop-blur transition-all duration-200 group-hover:opacity-100 hover:border-[var(--border-strong)] hover:text-[var(--text-strong)] focus-visible:opacity-100"
    >
      {copied ? (
        <>
          <svg
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M20 6 9 17l-5-5" />
          </svg>
          Copied
        </>
      ) : (
        <>
          <svg
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </svg>
          Copy
        </>
      )}
    </button>
  );
}
