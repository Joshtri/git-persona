import {
  ArrowRightFromSquare,
  BranchesDown,
  CircleCheck,
  Folder,
  Gear,
  Key,
  ListTimeline,
  Person,
} from "@gravity-ui/icons";
import type { ComponentType, SVGAttributes } from "react";
import { useEffect } from "react";
import { Avatar } from "@/components/avatar";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { Separator } from "@/components/separator";
import { Skeleton } from "@/components/skeleton";
import { SmartIdentityWidget } from "@/features/smart-switch";
import { cn } from "@/lib/cn";
// import { Avatar, Badge, Button, Separator, Skeleton } from "@/components/ui";
import { CREDENTIAL_HOST_VALUES, providerLabel } from "@/lib/credential-hosts";
import { useActivityStore } from "@/stores/activity";
import { useCredentialsStore } from "@/stores/credentials";
import { useProfilesStore } from "@/stores/profiles";
import { useReposStore } from "@/stores/repos";
import { useSshStore } from "@/stores/ssh";
import { useViewStore, type ViewName } from "@/stores/view";

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

function actionLabel(action: string): string {
  const map: Record<string, string> = {
    "profile.apply": "Applied profile globally",
    "profile.create": "Created profile",
    "profile.update": "Updated profile",
    "profile.delete": "Deleted profile",
    "ssh.import": "Imported SSH key",
    "ssh.generate": "Generated SSH key",
    "ssh.assign": "Assigned SSH key to profile",
    "ssh.remove": "Removed SSH key",
    "ssh.config.updated": "Updated SSH config",
    "credential.create": "Created credential",
    "credential.update": "Updated credential",
    "credential.assign": "Assigned credential to profile",
    "credential.switch": "Switched active credential",
    "credential.delete": "Deleted credential",
    "identity.auto_switch": "Auto-switched identity",
    "identity.same_profile": "Already on assigned profile",
    "identity.no_assignment": "Repository has no assigned profile",
    "identity.switch_cancelled": "Cancelled automatic switch",
  };
  return map[action] ?? action;
}

function SectionCard({ children, className }: { children: React.ReactNode; className?: string }) {
  return (
    <div
      className={cn(
        "rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) overflow-hidden",
        className
      )}
    >
      {children}
    </div>
  );
}

function CardHeader({ title, action }: { title: string; action?: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between px-4 pt-4 pb-3">
      <h2 className="text-xs font-semibold uppercase tracking-wider text-(--color-muted)">
        {title}
      </h2>
      {action}
    </div>
  );
}

interface QuickAction {
  label: string;
  Icon: ComponentType<SVGAttributes<SVGElement>>;
  view: ViewName;
}

const QUICK_ACTIONS: QuickAction[] = [
  { label: "Profiles", Icon: Person, view: "profiles" },
  { label: "SSH Keys", Icon: Key, view: "ssh" },
  { label: "Repos", Icon: Folder, view: "repos" },
  { label: "Activity", Icon: ListTimeline, view: "activity" },
  { label: "Settings", Icon: Gear, view: "settings" },
];

export function DashboardView() {
  const { navigate } = useViewStore();
  const { items: profiles, activeProfileId, loading: profilesLoading } = useProfilesStore();
  const { items: activity } = useActivityStore();
  const { items: repos } = useReposStore();
  const { items: sshKeys, fetch: fetchSsh } = useSshStore();
  const { items: credentials, fetch: fetchCredentials } = useCredentialsStore();

  useEffect(() => {
    fetchSsh();
    fetchCredentials();
  }, [fetchSsh, fetchCredentials]);

  const recentRepos = [...repos]
    .sort((a, b) => b.detected_at.localeCompare(a.detected_at))
    .slice(0, 3);
  const favoriteCount = repos.filter((r) => r.favorite).length;

  const sshAssigned = sshKeys.filter((k) => k.assigned_profile_id != null).length;
  const sshUnassigned = sshKeys.length - sshAssigned;
  const sshRecentImport = [...sshKeys]
    .filter((k) => k.imported)
    .sort((a, b) => b.created_at.localeCompare(a.created_at))[0];
  const activeSshKey = activeProfileId
    ? (sshKeys.find((k) => k.assigned_profile_id === activeProfileId) ?? null)
    : null;

  const credAssigned = credentials.filter((c) => c.profile_id != null).length;
  const credUnassigned = credentials.length - credAssigned;
  const credByProvider = CREDENTIAL_HOST_VALUES.map((host) => ({
    host,
    label: providerLabel(host),
    count: credentials.filter((c) => c.host === host).length,
  })).filter((p) => p.count > 0);

  const activeProfile = profiles.find((p) => p.id === activeProfileId) ?? null;

  const getProfileById = (id: string | undefined) =>
    id ? profiles.find((p) => p.id === id) : undefined;
  const getProfileColor = (id: string | undefined) =>
    id ? (getProfileById(id)?.color ?? "var(--color-brand-500)") : "var(--color-brand-500)";
  const getProfileLabel = (id: string | undefined) =>
    id ? (getProfileById(id)?.label ?? "Unknown") : "Unassigned";

  return (
    <div className="flex flex-col gap-4 max-w-3xl">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-(--color-fg)">Dashboard</h1>
          <p className="text-sm text-(--color-secondary) mt-0.5">
            {new Intl.DateTimeFormat(undefined, {
              weekday: "long",
              month: "long",
              day: "numeric",
            }).format(new Date())}
          </p>
        </div>
        <Badge variant="success">Git detected</Badge>
      </div>

      <SmartIdentityWidget />

      <div className="grid grid-cols-2 gap-4">
        {/* Active Profile */}
        <SectionCard
          className={
            activeProfile ? "border-(--color-brand-500)/30 bg-(--color-brand-500)/4" : undefined
          }
        >
          <CardHeader title="Active Profile" />
          <div className="px-4 pb-4 flex items-center gap-3">
            {profilesLoading && !activeProfile ? (
              <div className="flex gap-3 items-center w-full">
                <Skeleton className="size-10 rounded-full" />
                <div className="flex flex-col gap-1.5 flex-1">
                  <Skeleton className="h-4 w-24" />
                  <Skeleton className="h-3 w-32" />
                </div>
              </div>
            ) : activeProfile ? (
              <>
                <Avatar name={activeProfile.identity.name} color={activeProfile.color} size="xl" />
                <div className="flex flex-col gap-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-(--color-fg)">{activeProfile.label}</span>
                    <div className="size-2 rounded-full bg-(--color-success)" aria-hidden="true" />
                  </div>
                  <span className="text-sm text-(--color-secondary) truncate">
                    {activeProfile.identity.name}
                  </span>
                  <span className="text-xs text-(--color-muted) truncate">
                    {activeProfile.identity.email}
                  </span>
                  {activeProfile.identity.signing_key && (
                    <Badge variant="default" className="mt-1 w-fit">
                      <Key className="size-2.5 mr-1" aria-hidden="true" />
                      GPG signed
                    </Badge>
                  )}
                </div>
              </>
            ) : (
              <p className="text-sm text-(--color-muted)">No profile applied yet</p>
            )}
          </div>
          <Separator />
          <div className="px-4 py-3">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => navigate({ name: "profiles" })}
              className="gap-1.5"
            >
              <ArrowRightFromSquare className="size-3.5" aria-hidden="true" />
              Switch profile
            </Button>
          </div>
        </SectionCard>

        {/* System Status */}
        <SectionCard>
          <CardHeader title="System Status" />
          <div className="px-4 pb-4 flex flex-col gap-2.5">
            {[
              { label: "Git installation", status: "detected", ok: true },
              { label: "Global config", status: "~/.gitconfig", ok: true },
              {
                label: "SSH keys",
                status: `${sshKeys.length} managed`,
                ok: sshKeys.length > 0,
              },
              { label: "GPG agent", status: "inactive", ok: false },
            ].map(({ label, status, ok }) => (
              <div key={label} className="flex items-center justify-between text-sm">
                <span className="text-(--color-secondary)">{label}</span>
                <div className="flex items-center gap-1.5">
                  <span className={ok ? "text-(--color-success)" : "text-(--color-muted)"}>
                    {status}
                  </span>
                  <CircleCheck
                    className={cn(
                      "size-3.5",
                      ok ? "text-(--color-success)" : "text-(--color-muted)"
                    )}
                    aria-hidden="true"
                  />
                </div>
              </div>
            ))}
          </div>
        </SectionCard>
      </div>

      {/* Quick Actions */}
      <SectionCard>
        <CardHeader title="Quick Actions" />
        <div className="px-4 pb-4 flex gap-2 flex-wrap">
          {QUICK_ACTIONS.map(({ label, Icon, view }) => (
            <button
              key={label}
              type="button"
              onClick={() => navigate({ name: view })}
              className="flex items-center gap-2 px-3 py-2 rounded-(--radius-md) bg-(--color-surface-2) border border-(--color-border) text-sm text-(--color-secondary) hover:text-(--color-fg) hover:bg-(--color-surface-3) transition-colors"
            >
              <Icon className="size-3.5 shrink-0" aria-hidden="true" />
              {label}
            </button>
          ))}
        </div>
      </SectionCard>

      {/* SSH Identities */}
      <SectionCard>
        <CardHeader
          title="SSH Identities"
          action={
            <Button variant="ghost" size="sm" onClick={() => navigate({ name: "ssh" })}>
              Manage
            </Button>
          }
        />
        <div className="grid grid-cols-3 gap-px bg-(--color-border)">
          {[
            { label: "Total", value: sshKeys.length },
            { label: "Assigned", value: sshAssigned },
            { label: "Unassigned", value: sshUnassigned },
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
        <div className="px-4 py-3 flex items-center justify-between gap-3">
          <div className="flex items-center gap-2 min-w-0">
            <Key className="size-3.5 text-(--color-muted) shrink-0" aria-hidden="true" />
            <span className="text-xs text-(--color-secondary) shrink-0">Active identity</span>
            {activeSshKey ? (
              <span className="text-xs font-mono text-(--color-fg) truncate">
                {activeSshKey.label}
              </span>
            ) : (
              <span className="text-xs text-(--color-muted)">None for active profile</span>
            )}
          </div>
          {sshRecentImport && (
            <Badge variant="default" className="text-[10px] shrink-0">
              Recent: {sshRecentImport.label}
            </Badge>
          )}
        </div>
      </SectionCard>

      {/* HTTPS Credentials */}
      <SectionCard>
        <CardHeader
          title="HTTPS Credentials"
          action={
            <Button variant="ghost" size="sm" onClick={() => navigate({ name: "credentials" })}>
              Manage
            </Button>
          }
        />
        <div className="grid grid-cols-3 gap-px bg-(--color-border)">
          {[
            { label: "Total", value: credentials.length },
            { label: "Assigned", value: credAssigned },
            { label: "Unassigned", value: credUnassigned },
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
        <div className="px-4 py-3 flex items-center gap-2 flex-wrap min-h-11">
          <span className="text-xs text-(--color-secondary) shrink-0">By provider</span>
          {credByProvider.length === 0 ? (
            <span className="text-xs text-(--color-muted)">No credentials yet</span>
          ) : (
            credByProvider.map((p) => (
              <Badge key={p.host} variant="default" className="text-[10px]">
                {p.label} · {p.count}
              </Badge>
            ))
          )}
        </div>
      </SectionCard>

      <div className="grid grid-cols-2 gap-4">
        {/* Recent Activity */}
        <SectionCard>
          <CardHeader
            title="Recent Activity"
            action={
              <Button variant="ghost" size="sm" onClick={() => navigate({ name: "activity" })}>
                View all
              </Button>
            }
          />
          <div className="flex flex-col">
            {activity.length === 0 ? (
              <p className="px-4 pb-4 text-sm text-(--color-muted)">No activity yet</p>
            ) : (
              activity.slice(0, 4).map((entry, i) => {
                const profile = getProfileById(entry.profile_id);
                return (
                  <div
                    key={entry.id}
                    className={cn(
                      "flex items-start gap-3 px-4 py-2.5 text-sm",
                      i > 0 && "border-t border-(--color-border)"
                    )}
                  >
                    {profile && (
                      <Avatar
                        name={profile.identity.name}
                        color={profile.color}
                        size="sm"
                        className="mt-0.5 shrink-0"
                      />
                    )}
                    <div className="flex flex-col gap-0.5 flex-1 min-w-0">
                      <span className="text-(--color-fg) text-xs">{actionLabel(entry.action)}</span>
                      {profile && (
                        <span className="text-[10px] text-(--color-muted)">{profile.label}</span>
                      )}
                    </div>
                    <span className="text-[10px] text-(--color-muted) shrink-0">
                      {timeAgo(entry.timestamp)}
                    </span>
                  </div>
                );
              })
            )}
          </div>
        </SectionCard>

        {/* Profiles */}
        <SectionCard>
          <CardHeader
            title="Profiles"
            action={
              <Button variant="ghost" size="sm" onClick={() => navigate({ name: "profiles" })}>
                View all
              </Button>
            }
          />
          <div className="flex flex-col">
            {profiles.length === 0 ? (
              <p className="px-4 pb-4 text-sm text-(--color-muted)">No profiles yet</p>
            ) : (
              profiles.slice(0, 4).map((profile, i) => (
                <div
                  key={profile.id}
                  className={cn(
                    "flex items-center gap-3 px-4 py-2.5",
                    i > 0 && "border-t border-(--color-border)",
                    profile.id === activeProfileId && "bg-(--color-brand-500)/5"
                  )}
                >
                  <Avatar name={profile.identity.name} color={profile.color} size="sm" />
                  <div className="flex flex-col gap-0.5 min-w-0 flex-1">
                    <span className="text-xs font-medium text-(--color-fg)">{profile.label}</span>
                    <span className="text-[10px] text-(--color-muted) truncate">
                      {profile.identity.email}
                    </span>
                  </div>
                  {profile.id === activeProfileId && (
                    <Badge variant="success" className="text-[10px]">
                      Active
                    </Badge>
                  )}
                </div>
              ))
            )}
          </div>
        </SectionCard>
      </div>

      {/* Recent Repositories */}
      <SectionCard>
        <CardHeader
          title={`Recent Repositories · ${repos.length}${favoriteCount ? ` · ${favoriteCount} ★` : ""}`}
          action={
            <Button variant="ghost" size="sm" onClick={() => navigate({ name: "repos" })}>
              View all
            </Button>
          }
        />
        {recentRepos.length === 0 ? (
          <p className="px-4 pb-4 text-sm text-(--color-muted)">
            No repositories yet — scan folders to discover them.
          </p>
        ) : (
          <div className="grid grid-cols-3 gap-px bg-(--color-border)">
            {recentRepos.map((repo) => (
              <div key={repo.id} className="flex flex-col gap-1.5 p-3.5 bg-(--color-surface)">
                <div className="flex items-center gap-2">
                  <Folder className="size-3.5 text-(--color-muted) shrink-0" aria-hidden="true" />
                  <span className="text-sm font-medium text-(--color-fg) truncate">
                    {repo.name}
                  </span>
                </div>
                <div className="flex items-center gap-1.5">
                  <BranchesDown
                    className="size-3 text-(--color-muted) shrink-0"
                    aria-hidden="true"
                  />
                  <span className="text-xs text-(--color-muted) font-mono">
                    {repo.active_branch ?? "detached"}
                  </span>
                </div>
                <div className="flex items-center gap-1.5 mt-0.5">
                  <div
                    className="size-2 rounded-full shrink-0"
                    style={{
                      backgroundColor: getProfileColor(repo.active_profile_id ?? undefined),
                    }}
                  />
                  <span className="text-[10px] text-(--color-muted)">
                    {getProfileLabel(repo.active_profile_id ?? undefined)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </SectionCard>
    </div>
  );
}
