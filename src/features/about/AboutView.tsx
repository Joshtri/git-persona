import { ArrowUpRightFromSquare, LogoGithub, Star } from "@gravity-ui/icons";
import { Badge } from "@/components/badge";
import { BrandLogo } from "@/components/brand-logo";
import { Button } from "@/components/button";
import { Separator } from "@/components/separator";
import { useAppVersion } from "@/hooks/useAppVersion";
import { openUrl } from "@/ipc/client";
import { osDisplayName } from "@/lib/platform";
import { usePlatformStore } from "@/stores/platform";

const GITHUB_URL = "https://github.com/Joshtri/git-persona";

const CHANNEL = (import.meta.env.VITE_CHANNEL ?? "stable") as string;

const CHANNEL_BADGE: Record<string, "success" | "warning" | "danger" | "default"> = {
  stable: "success",
  beta: "warning",
  alpha: "warning",
  nightly: "danger",
};

const OSS_DEPS = [
  { name: "Tauri", license: "MIT / Apache-2.0", description: "Desktop runtime" },
  { name: "React", license: "MIT", description: "UI framework" },
  { name: "Tailwind CSS", license: "MIT", description: "Styling" },
  { name: "Zustand", license: "MIT", description: "State management" },
  { name: "motion", license: "MIT", description: "Animations" },
  { name: "Base UI", license: "MIT", description: "Headless UI primitives" },
  { name: "Gravity UI Icons", license: "MIT", description: "Icon set" },
  { name: "Biome", license: "MIT", description: "Linting & formatting" },
  { name: "gix", license: "MIT / Apache-2.0", description: "Git backend (Rust)" },
];

export function AboutView() {
  const version = useAppVersion();
  const capabilities = usePlatformStore((s) => s.capabilities);

  return (
    <div className="flex flex-col gap-8 max-w-lg">
      {/* Hero card */}
      <div className="rounded-(--radius-xl) border border-(--color-border) bg-(--color-surface) p-6 flex flex-col items-center gap-4 text-center">
        <div className="p-3 rounded-(--radius-xl) bg-(--color-surface-2) border border-(--color-border)">
          <BrandLogo size="xl" />
        </div>

        <div className="flex flex-col gap-1.5">
          <h1 className="text-2xl font-bold text-(--color-fg) tracking-tight">GitPersona</h1>
          <p className="text-sm text-(--color-secondary)">Developer Identity Manager</p>
        </div>

        <div className="flex items-center gap-2">
          <Badge variant="success">v{version}</Badge>
          <Badge variant={CHANNEL_BADGE[CHANNEL] ?? "default"}>
            {CHANNEL.charAt(0).toUpperCase() + CHANNEL.slice(1)}
          </Badge>
        </div>

        <p className="text-sm text-(--color-secondary) leading-relaxed max-w-sm">
          Manage multiple Git identities — name, email, and signing keys — and switch between them
          globally or per-repository. Built for developers who maintain separate work, personal, and
          open-source profiles.
        </p>

        {/* Action links */}
        <div className="flex flex-wrap justify-center gap-2 pt-1">
          <Button
            variant="secondary"
            size="sm"
            className="gap-1.5"
            onClick={() => openUrl(GITHUB_URL)}
          >
            <LogoGithub className="size-3.5" aria-hidden="true" />
            View on GitHub
            <ArrowUpRightFromSquare className="size-3 text-(--color-muted)" aria-hidden="true" />
          </Button>
          <Button
            variant="secondary"
            size="sm"
            className="gap-1.5"
            onClick={() => openUrl(GITHUB_URL)}
          >
            <Star className="size-3.5" aria-hidden="true" />
            Star on GitHub
          </Button>
        </div>
      </div>

      {/* Build info */}
      <section className="flex flex-col gap-3">
        <h2 className="text-xs font-semibold uppercase tracking-wider text-(--color-muted)">
          Build Info
        </h2>
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
          {(
            [
              ["Version", `v${version}`],
              ["Platform", capabilities ? osDisplayName(capabilities.os) : "—"],
              ["Runtime", "Tauri 2"],
              ["Frontend", "React 19"],
            ] as const
          ).map(([k, v]) => (
            <div
              key={k}
              className="flex items-center justify-between px-4 py-2.5 border-b border-(--color-border) last:border-b-0"
            >
              <span className="text-sm text-(--color-secondary)">{k}</span>
              <span className="text-sm text-(--color-fg) font-mono">{v}</span>
            </div>
          ))}
        </div>
      </section>

      <Separator />

      {/* Open source dependencies */}
      <section className="flex flex-col gap-3">
        <div className="flex items-baseline justify-between">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-(--color-muted)">
            Open Source
          </h2>
          <span className="text-xs text-(--color-muted)">Built on the shoulders of giants</span>
        </div>
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
              <Badge variant="default" className="text-[10px] shrink-0">
                {dep.license}
              </Badge>
            </div>
          ))}
        </div>
      </section>

      {/* Footer */}
      <div className="flex flex-col items-center gap-1.5 pb-2">
        <p className="text-xs text-(--color-muted)">
          MIT License — Copyright © {new Date().getFullYear()} Joshtri
        </p>
        <p className="text-xs text-(--color-muted)">
          Made by{" "}
          <button
            type="button"
            onClick={() => openUrl("https://github.com/Joshtri")}
            className="text-(--color-brand-400) hover:text-(--color-brand-300) hover:underline transition-colors cursor-pointer"
          >
            Joshtri
          </button>
          {" · "}
          <button
            type="button"
            onClick={() => openUrl("https://joshtrilenggu.com")}
            className="text-(--color-brand-400) hover:text-(--color-brand-300) hover:underline transition-colors cursor-pointer"
          >
            joshtrilenggu.com
          </button>
        </p>
      </div>
    </div>
  );
}
