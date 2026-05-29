import { Section, SectionHeading } from "./Section";
import { RevealGroup, RevealItem } from "./Reveal";

const groups = [
  {
    label: "Files & edits",
    tools: ["read_file", "write_file", "edit_file", "apply_patch", "spawn_subagent"],
  },
  {
    label: "Search",
    tools: ["glob_tool", "grep_tool", "list_dir", "semantic_search"],
  },
  {
    label: "Shell & web",
    tools: ["run_shell", "web_fetch", "web_search"],
  },
  {
    label: "Planning",
    tools: ["present_plan"],
  },
  {
    label: "Learning",
    tools: [
      "save_memory",
      "recall_memory",
      "forget_memory",
      "invoke_skill",
      "recall_session",
      "read_session",
    ],
  },
];

export function ToolsSection() {
  return (
    <Section id="tools">
      <SectionHeading
        eyebrow="Built-in tools"
        title="A complete tool layer, gated by permission mode."
        lead="Each tool runs through a registry that carries the active permission mode, working directory, and project root — so it can act, prompt, or refuse based on context."
      />
      <RevealGroup className="mt-12 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {groups.map((g) => (
          <RevealItem key={g.label}>
            <div className="h-full rounded-2xl border border-[var(--border)] bg-[var(--surface)] p-6">
              <h3 className="text-xs font-bold uppercase tracking-wider text-[var(--text-dim)]">
                {g.label}
              </h3>
              <ul className="mt-4 flex flex-wrap gap-2">
                {g.tools.map((t) => (
                  <li
                    key={t}
                    className="rounded-md border border-[var(--border)] bg-[var(--color-carbon-300)] px-2.5 py-1 font-mono text-xs text-[var(--color-french-600)]"
                  >
                    {t}
                  </li>
                ))}
              </ul>
            </div>
          </RevealItem>
        ))}
      </RevealGroup>
    </Section>
  );
}
