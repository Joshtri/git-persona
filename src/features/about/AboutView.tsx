import { ArrowUpRightFromSquare, LogoGithub, Star } from "@gravity-ui/icons";
import { Badge } from "@/components/badge";
import { BrandLogo } from "@/components/brand-logo";
import { Button } from "@/components/button";
import { Separator } from "@/components/separator";
import { useAppVersion } from "@/hooks/useAppVersion";

// import { Badge, Button, Separator } from "@/components/ui";

const OSS_DEPS = [
  { name: "Tauri", license: "MIT / Apache-2.0", description: "Desktop runtime" },
  { name: "React", license: "MIT", description: "UI framework" },
  { name: "Tailwind CSS", license: "MIT", description: "Styling" },
  { name: "Zustand", license: "MIT", description: "State management" },
  { name: "Zod", license: "MIT", description: "Schema validation" },
  { name: "react-hook-form", license: "MIT", description: "Form handling" },
  { name: "motion", license: "MIT", description: "Animations" },
  { name: "Base UI", license: "MIT", description: "Headless UI primitives" },
  { name: "Gravity UI Icons", license: "MIT", description: "Icon set" },
  { name: "Biome", license: "MIT", description: "Linting & formatting" },
];

export function AboutView() {
  const version = useAppVersion();
  return (
    <div className="flex flex-col gap-6 max-w-lg">
      {/* App identity */}
      <div className="flex items-center gap-5">
        <BrandLogo size="xl" />
        <div className="flex flex-col gap-1">
          <h1 className="text-xl font-bold text-(--color-fg) tracking-tight">GitPersona</h1>
          <p className="text-sm text-(--color-secondary)">Developer Identity Manager</p>
          <div className="flex items-center gap-2 mt-1">
            <Badge variant="default">v{version}</Badge>
            <Badge variant="default">Sprint 1</Badge>
          </div>
        </div>
      </div>

      <Separator />

      {/* Description */}
      <div className="flex flex-col gap-2">
        <p className="text-sm text-(--color-secondary) leading-relaxed">
          GitPersona lets developers manage multiple Git identities — name, email, and signing keys
          — and switch between them globally or per-repository. Built for developers who maintain
          separate work, personal, and open-source identities.
        </p>
      </div>

      {/* Links */}
      <div className="flex gap-3">
        <Button variant="secondary" size="sm" disabled className="gap-1.5">
          <LogoGithub className="size-3.5" aria-hidden="true" />
          GitHub
          <ArrowUpRightFromSquare className="size-3 text-(--color-muted)" aria-hidden="true" />
        </Button>
        <Button variant="secondary" size="sm" disabled className="gap-1.5">
          <ArrowUpRightFromSquare className="size-3.5" aria-hidden="true" />
          Website
        </Button>
        <Button variant="secondary" size="sm" disabled className="gap-1.5">
          <Star className="size-3.5" aria-hidden="true" />
          Star on GitHub
        </Button>
      </div>

      <Separator />

      {/* Build info */}
      <div className="flex flex-col gap-3">
        <h2 className="text-xs font-semibold uppercase tracking-wider text-(--color-muted)">
          Build Info
        </h2>
        <div className="grid grid-cols-2 gap-2 text-sm">
          {[
            ["Version", version],
            ["Platform", "Windows 11"],
            ["Runtime", "Tauri 2"],
            ["Frontend", "React 19"],
            ["Build", "Debug"],
            ["Node", "22.x"],
          ].map(([k, v]) => (
            <div key={k} className="flex justify-between">
              <span className="text-(--color-muted)">{k}</span>
              <span className="text-(--color-secondary) font-mono text-xs">{v}</span>
            </div>
          ))}
        </div>
      </div>

      <Separator />

      {/* License */}
      <div className="flex flex-col gap-2">
        <h2 className="text-xs font-semibold uppercase tracking-wider text-(--color-muted)">
          License
        </h2>
        <p className="text-sm text-(--color-secondary)">
          MIT License — Copyright © 2025 GitPersona Contributors
        </p>
      </div>

      {/* Open source */}
      <div className="flex flex-col gap-3">
        <h2 className="text-xs font-semibold uppercase tracking-wider text-(--color-muted)">
          Open Source
        </h2>
        <p className="text-xs text-(--color-muted)">
          GitPersona is built on the shoulders of giants.
        </p>
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
          {OSS_DEPS.map((dep) => (
            <div
              key={dep.name}
              className="flex items-center justify-between px-4 py-2.5 border-b border-(--color-border) last:border-b-0"
            >
              <div className="flex flex-col gap-0.5">
                <span className="text-sm font-medium text-(--color-fg)">{dep.name}</span>
                <span className="text-xs text-(--color-muted)">{dep.description}</span>
              </div>
              <Badge variant="default" className="text-[10px]">
                {dep.license}
              </Badge>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
