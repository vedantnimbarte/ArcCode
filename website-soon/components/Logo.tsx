export function Logo({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 32 32"
      className={className}
      fill="none"
      aria-hidden
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect width="32" height="32" rx="7" fill="var(--color-carbon-300)" />
      <path
        d="M9 21 L16 8 L23 21"
        stroke="var(--text-strong)"
        strokeWidth="2.4"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M11.5 17 h9"
        stroke="var(--color-french-500)"
        strokeWidth="2.4"
        strokeLinecap="round"
      />
    </svg>
  );
}
