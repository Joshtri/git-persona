import { ArrowRotateLeft, CircleCheck, Gear, Person } from "@gravity-ui/icons";
import { useEffect } from "react";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { Separator } from "@/components/separator";
import { Switch } from "@/components/switch";
import { cn } from "@/lib/cn";
import { useSmartSwitchStore } from "@/stores/smartSwitch";
import { useViewStore } from "@/stores/view";

function timeAgo(iso: string): string {
  const mins = Math.floor((Date.now() - new Date(iso).getTime()) / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

function formatUptime(iso: string): string {
  const secs = Math.floor((Date.now() - new Date(iso).getTime()) / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ${mins % 60}m`;
  return `${Math.floor(hours / 24)}d ${hours % 24}h`;
}

export function SmartIdentityWidget() {
  const { navigate } = useViewStore();
  const { status, init, setEnabled, pause, resume, restart } = useSmartSwitchStore();

  useEffect(() => {
    init();
  }, [init]);

  const enabled = status?.enabled ?? false;
  const watching = status?.watching ?? false;
  const paused = status?.paused ?? false;
  const last = status?.last_switch ?? null;

  const state = !enabled
    ? ({ label: "Off", variant: "default" } as const)
    : paused
      ? ({ label: "Paused", variant: "warning" } as const)
      : watching
        ? ({ label: "Watching", variant: "success" } as const)
        : ({ label: "Idle", variant: "default" } as const);

  return (
    <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden">
      <div className="flex items-center justify-between px-4 pt-4 pb-3">
        <div className="flex items-center gap-2">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-(--color-muted)">
            Smart Identity
          </h2>
          <Badge variant={state.variant}>{state.label}</Badge>
        </div>
        <Switch checked={enabled} onCheckedChange={(v) => setEnabled(v)} />
      </div>

      <div className="grid grid-cols-3 gap-px bg-(--color-border)">
        {[
          { label: "Monitored", value: String(status?.repos_monitored ?? 0) },
          {
            label: "Uptime",
            value: status?.started_at ? formatUptime(status.started_at) : "—",
          },
          {
            label: "State",
            value: !enabled ? "Off" : paused ? "Paused" : watching ? "Live" : "Idle",
          },
        ].map((stat) => (
          <div
            key={stat.label}
            className="flex flex-col gap-0.5 p-3.5 bg-(--color-surface) items-center"
          >
            <span className="text-lg font-semibold text-(--color-fg)">{stat.value}</span>
            <span className="text-[10px] uppercase tracking-wider text-(--color-muted)">
              {stat.label}
            </span>
          </div>
        ))}
      </div>

      <Separator />

      <div className="px-4 py-3 flex items-center justify-between gap-3 min-h-11">
        <div className="flex items-center gap-2 min-w-0">
          {last ? (
            <>
              <CircleCheck
                className="size-3.5 text-(--color-success) shrink-0"
                aria-hidden="true"
              />
              <span className="text-xs text-(--color-secondary) shrink-0">Last switch</span>
              <span className="text-xs font-medium text-(--color-fg) truncate">
                {last.profile_label ?? "Unknown"}
              </span>
              <span className="text-[10px] text-(--color-muted) shrink-0">{timeAgo(last.at)}</span>
            </>
          ) : (
            <>
              <Person className="size-3.5 text-(--color-muted) shrink-0" aria-hidden="true" />
              <span className="text-xs text-(--color-muted)">No automatic switch yet</span>
            </>
          )}
        </div>
      </div>

      <Separator />

      <div className="px-4 py-3 flex items-center gap-2 flex-wrap">
        {enabled &&
          watching &&
          (paused ? (
            <Button variant="ghost" size="sm" onClick={() => resume()}>
              Resume
            </Button>
          ) : (
            <Button variant="ghost" size="sm" onClick={() => pause()}>
              Pause
            </Button>
          ))}
        {enabled && (
          <Button variant="ghost" size="sm" onClick={() => restart()} className="gap-1.5">
            <ArrowRotateLeft className="size-3.5" aria-hidden="true" />
            Restart
          </Button>
        )}
        <Button
          variant="ghost"
          size="sm"
          onClick={() => navigate({ name: "settings" })}
          className={cn("gap-1.5", enabled ? "ml-auto" : "")}
        >
          <Gear className="size-3.5" aria-hidden="true" />
          Configure
        </Button>
      </div>
    </div>
  );
}
