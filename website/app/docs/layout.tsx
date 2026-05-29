import type { Metadata } from "next";
import { DocsSidebar } from "@/components/DocsSidebar";
import { TableOfContents } from "@/components/TableOfContents";
import { DocsPager } from "@/components/DocsPager";

export const metadata: Metadata = {
  title: "Documentation",
};

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="container-page pt-28 pb-24">
      <div className="grid gap-x-10 gap-y-6 lg:grid-cols-[14rem_minmax(0,1fr)] xl:grid-cols-[14rem_minmax(0,1fr)_14rem]">
        <DocsSidebar />
        <article id="docs-content" className="min-w-0 max-w-3xl">
          {children}
          <DocsPager />
        </article>
        <TableOfContents />
      </div>
    </div>
  );
}
