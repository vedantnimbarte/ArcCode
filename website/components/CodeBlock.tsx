import { CopyButton } from "./CopyButton";

type Line = { text: string; tone?: "prompt" | "comment" | "out" | "default" };

const toneClass: Record<NonNullable<Line["tone"]>, string> = {
  prompt: "text-[var(--text-strong)]",
  comment: "text-[var(--text-dim)]",
  out: "text-[var(--color-french-400)]",
  default: "text-[var(--text-muted)]",
};

/**
 * Lightweight static code block for the landing page (no Shiki). Each line can
 * carry a tone for a terminal-transcript look.
 */
export function CodeBlock({
  lines,
  title,
  copyText,
}: {
  lines: Line[];
  title?: string;
  copyText?: string;
}) {
  const text = copyText ?? lines.map((l) => l.text).join("\n");
  return (
    <div className="group relative overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--color-carbon-200)]">
      {title ? (
        <div className="flex items-center gap-2 border-b border-[var(--border)] bg-[var(--color-carbon-300)] px-4 py-2">
          <span className="font-mono text-[0.7rem] uppercase tracking-wider text-[var(--text-dim)]">
            {title}
          </span>
        </div>
      ) : null}
      <CopyButton text={text} />
      <pre className="overflow-x-auto px-4 py-4 font-mono text-[0.82rem] leading-relaxed">
        <code>
          {lines.map((l, i) => (
            <span
              key={i}
              className={`block ${toneClass[l.tone ?? "default"]}`}
            >
              {l.text || " "}
            </span>
          ))}
        </code>
      </pre>
    </div>
  );
}
