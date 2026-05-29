import type { ReactNode } from "react";
import { Reveal } from "./Reveal";

export function Eyebrow({ children }: { children: ReactNode }) {
  return (
    <span className="inline-flex items-center gap-2 rounded-full border border-[var(--border)] bg-[var(--color-carbon-300)] px-3 py-1 font-mono text-[0.7rem] font-medium uppercase tracking-[0.14em] text-[var(--text-muted)]">
      <span className="h-1.5 w-1.5 rounded-full bg-[var(--color-french-500)]" />
      {children}
    </span>
  );
}

export function SectionHeading({
  eyebrow,
  title,
  lead,
  align = "left",
}: {
  eyebrow: string;
  title: ReactNode;
  lead?: ReactNode;
  align?: "left" | "center";
}) {
  return (
    <Reveal
      className={`max-w-2xl ${align === "center" ? "mx-auto text-center" : ""}`}
    >
      <Eyebrow>{eyebrow}</Eyebrow>
      <h2 className="mt-5 text-balance text-3xl font-extrabold tracking-tight text-[var(--text-strong)] sm:text-4xl">
        {title}
      </h2>
      {lead ? (
        <p className="mt-4 text-pretty text-base leading-7 text-[var(--text-muted)] sm:text-lg">
          {lead}
        </p>
      ) : null}
    </Reveal>
  );
}

export function Section({
  id,
  children,
  className,
}: {
  id?: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section
      id={id}
      className={`container-page scroll-mt-24 py-20 sm:py-28 ${className ?? ""}`}
    >
      {children}
    </section>
  );
}
