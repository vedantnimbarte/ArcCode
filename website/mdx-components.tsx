import type { MDXComponents } from "mdx/types";
import Link from "next/link";
import { CopyButton } from "@/components/CopyButton";

// Maps markdown elements to themed components for all .mdx docs pages.
export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    h1: (props) => (
      <h1
        className="mt-2 mb-4 scroll-mt-28 text-4xl font-extrabold tracking-tight text-[var(--text-strong)]"
        {...props}
      />
    ),
    h2: (props) => (
      <h2
        className="mt-14 mb-4 scroll-mt-28 border-t border-[var(--border)] pt-8 text-2xl font-bold tracking-tight text-[var(--text-strong)]"
        {...props}
      />
    ),
    h3: (props) => (
      <h3
        className="mt-9 mb-3 scroll-mt-28 text-lg font-bold text-[var(--text-strong)]"
        {...props}
      />
    ),
    h4: (props) => (
      <h4
        className="mt-6 mb-2 scroll-mt-28 text-base font-bold text-[var(--text)]"
        {...props}
      />
    ),
    p: (props) => (
      <p className="my-4 leading-7 text-[var(--text-muted)]" {...props} />
    ),
    a: ({ href = "", ...props }) => {
      const external = /^https?:\/\//.test(href);
      return (
        <Link
          href={href}
          className="font-medium text-[var(--color-french-700)] underline decoration-[var(--border-strong)] underline-offset-4 transition-colors hover:text-[var(--text-strong)] hover:decoration-[var(--color-french-500)]"
          {...(external ? { target: "_blank", rel: "noreferrer" } : {})}
          {...props}
        />
      );
    },
    ul: (props) => (
      <ul
        className="my-4 ml-1 list-none space-y-2 text-[var(--text-muted)]"
        {...props}
      />
    ),
    ol: (props) => (
      <ol
        className="my-4 ml-5 list-decimal space-y-2 text-[var(--text-muted)] marker:text-[var(--text-dim)]"
        {...props}
      />
    ),
    li: (props) => (
      <li
        className="leading-7 [ul>&]:relative [ul>&]:pl-5 [ul>&]:before:absolute [ul>&]:before:left-0 [ul>&]:before:top-3 [ul>&]:before:h-1.5 [ul>&]:before:w-1.5 [ul>&]:before:rounded-full [ul>&]:before:bg-[var(--color-french-400)]"
        {...props}
      />
    ),
    blockquote: (props) => (
      <blockquote
        className="my-6 border-l-2 border-[var(--color-french-500)] bg-[var(--color-carbon-300)] py-2 pl-5 pr-4 text-[var(--text-muted)] italic"
        {...props}
      />
    ),
    hr: () => <hr className="my-10 border-[var(--border)]" />,
    strong: (props) => (
      <strong className="font-bold text-[var(--text-strong)]" {...props} />
    ),
    table: (props) => (
      <div className="my-7 overflow-x-auto rounded-xl border border-[var(--border)]">
        <table className="w-full border-collapse text-left text-sm" {...props} />
      </div>
    ),
    thead: (props) => (
      <thead
        className="bg-[var(--color-carbon-300)] text-[var(--text-strong)]"
        {...props}
      />
    ),
    th: (props) => (
      <th
        className="border-b border-[var(--border)] px-4 py-3 font-semibold whitespace-nowrap"
        {...props}
      />
    ),
    td: (props) => (
      <td
        className="border-b border-[var(--border)] px-4 py-3 align-top text-[var(--text-muted)]"
        {...props}
      />
    ),
    pre: ({ children, ...props }) => (
      <div className="group relative my-6">
        <CopyButton />
        <pre {...props}>{children}</pre>
      </div>
    ),
    ...components,
  };
}
