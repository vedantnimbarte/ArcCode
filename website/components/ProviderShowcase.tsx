import { Section, SectionHeading } from "./Section";
import { Reveal } from "./Reveal";

type Tier = "native" | "compat" | "untested";

const tierMeta: Record<Tier, { label: string; dot: string }> = {
  native: { label: "Native adapter", dot: "bg-[var(--color-snow)]" },
  compat: { label: "OpenAI-compatible", dot: "bg-[var(--color-french-500)]" },
  untested: { label: "Untested / local", dot: "bg-[var(--color-iron-500)]" },
};

const providers: { name: string; tier: Tier }[] = [
  { name: "Anthropic", tier: "native" },
  { name: "Google Gemini", tier: "native" },
  { name: "IBM watsonx", tier: "native" },
  { name: "Cohere", tier: "native" },
  { name: "OpenAI", tier: "compat" },
  { name: "ChatGPT (OAuth)", tier: "compat" },
  { name: "OpenRouter", tier: "compat" },
  { name: "Groq", tier: "compat" },
  { name: "Together AI", tier: "compat" },
  { name: "Fireworks", tier: "compat" },
  { name: "DeepSeek", tier: "compat" },
  { name: "Mistral", tier: "compat" },
  { name: "xAI Grok", tier: "compat" },
  { name: "Cerebras", tier: "compat" },
  { name: "Azure OpenAI", tier: "compat" },
  { name: "AWS Bedrock", tier: "compat" },
  { name: "GCP Vertex AI", tier: "compat" },
  { name: "LiteLLM", tier: "compat" },
  { name: "Perplexity", tier: "untested" },
  { name: "Ollama", tier: "untested" },
  { name: "LM Studio", tier: "untested" },
  { name: "vLLM", tier: "untested" },
  { name: "llama.cpp", tier: "untested" },
  { name: "DeepInfra", tier: "compat" },
  { name: "SambaNova", tier: "compat" },
  { name: "NVIDIA NIM", tier: "compat" },
  { name: "Hyperbolic", tier: "compat" },
  { name: "Nebius", tier: "compat" },
];

function Pill({ name, tier }: { name: string; tier: Tier }) {
  return (
    <span className="inline-flex shrink-0 items-center gap-2 rounded-full border border-[var(--border)] bg-[var(--surface)] px-4 py-2 text-sm font-medium text-[var(--text)]">
      <span className={`h-1.5 w-1.5 rounded-full ${tierMeta[tier].dot}`} />
      {name}
    </span>
  );
}

function Row({
  items,
  reverse,
}: {
  items: { name: string; tier: Tier }[];
  reverse?: boolean;
}) {
  return (
    <div className="group flex overflow-hidden [mask-image:linear-gradient(90deg,transparent,#000_12%,#000_88%,transparent)]">
      <div
        className="flex shrink-0 gap-3 pr-3 group-hover:[animation-play-state:paused]"
        style={{
          animation: reverse
            ? "var(--animate-marquee-rev)"
            : "var(--animate-marquee)",
        }}
      >
        {[...items, ...items].map((p, i) => (
          <Pill key={`${p.name}-${i}`} {...p} />
        ))}
      </div>
    </div>
  );
}

export function ProviderShowcase() {
  const half = Math.ceil(providers.length / 2);
  return (
    <Section id="providers">
      <SectionHeading
        eyebrow="Providers"
        title="73+ providers. One streaming interface."
        lead="Anthropic is the reference implementation. A single OpenAI-compatible adapter covers dozens of hosted and local backends — swap any of them in with one flag."
      />

      <Reveal className="mt-12 flex flex-col gap-3" delay={0.05}>
        <Row items={providers.slice(0, half)} />
        <Row items={providers.slice(half)} reverse />
      </Reveal>

      <Reveal className="mt-8 flex flex-wrap items-center gap-x-6 gap-y-2" delay={0.1}>
        {(Object.keys(tierMeta) as Tier[]).map((t) => (
          <span
            key={t}
            className="inline-flex items-center gap-2 text-sm text-[var(--text-dim)]"
          >
            <span className={`h-2 w-2 rounded-full ${tierMeta[t].dot}`} />
            {tierMeta[t].label}
          </span>
        ))}
        <span className="text-sm text-[var(--text-dim)]">
          …and many more — see the full table in the docs.
        </span>
      </Reveal>
    </Section>
  );
}
