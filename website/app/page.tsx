import { Hero } from "@/components/Hero";
import { FeatureGrid } from "@/components/FeatureGrid";
import { ProviderShowcase } from "@/components/ProviderShowcase";
import { PilotTiers } from "@/components/PilotTiers";
import { LearningLoop } from "@/components/LearningLoop";
import { ToolsSection } from "@/components/ToolsSection";
import { Comparison } from "@/components/Comparison";
import { InstallSection } from "@/components/InstallSection";

export default function HomePage() {
  return (
    <>
      <Hero />
      <FeatureGrid />
      <ProviderShowcase />
      <PilotTiers />
      <LearningLoop />
      <ToolsSection />
      <Comparison />
      <InstallSection />
    </>
  );
}
