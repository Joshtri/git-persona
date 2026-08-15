import { zodResolver } from "@hookform/resolvers/zod";
import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import { z } from "zod";
import { Badge } from "@/components/badge";
import { Button } from "@/components/button";
import { ErrorState } from "@/components/feedback/ErrorState";
import { LoadingState } from "@/components/feedback/LoadingState";
import { Separator } from "@/components/separator";
import { Switch } from "@/components/switch";
import { useAppVersion } from "@/hooks/useAppVersion";
// import { Badge, Button, Separator, Switch } from "@/components/ui";
import type { AppSettings, Theme } from "@/ipc/types.gen";
import { useFeedbackStore } from "@/stores/feedback";
import { useSettingsStore } from "@/stores/settings";
import { useSmartSwitchStore } from "@/stores/smartSwitch";
import { Select, type SelectOption } from "../inputs/non-form/select";
import { SectionHeader } from "./components/Section.Header";
import { SettingRow } from "./components/Section.SettingRow";

const schema = z.object({
  theme: z.enum(["Dark", "Light", "System"]),
  show_audit_log: z.boolean(),
  auto_scan_repos: z.boolean(),
  launch_at_startup: z.boolean(),
  start_minimized: z.boolean(),
  close_to_tray: z.boolean(),
});

type FormData = z.infer<typeof schema>;
const THEMES: Theme[] = ["Dark", "Light", "System"];

const THEME_OPTIONS: SelectOption[] = THEMES.map((theme) => ({
  label: theme,
  value: theme,
}));

const DEFAULT_SMART = {
  enabled: false,
  auto_ssh: true,
  auto_credential: true,
  show_notification: true,
  confirm_before_switch: false,
  start_on_launch: false,
};

export function SettingsView() {
  const version = useAppVersion();
  const { data, loading, error, fetch, update } = useSettingsStore();
  const toast = useFeedbackStore((s) => s.toast);
  const setSmartEnabled = useSmartSwitchStore((s) => s.setEnabled);
  const patchSmart = useSmartSwitchStore((s) => s.patchSettings);
  const smart = data?.smart_switching ?? DEFAULT_SMART;

  useEffect(() => {
    fetch();
  }, [fetch]);

  const { control, handleSubmit, reset, watch, setValue } = useForm<FormData>({
    resolver: zodResolver(schema),
    defaultValues: data ?? {
      theme: "Dark",
      show_audit_log: true,
      auto_scan_repos: false,
      launch_at_startup: false,
      start_minimized: false,
      close_to_tray: false,
    },
  });

  useEffect(() => {
    if (data) reset(data);
  }, [data, reset]);

  const selectedTheme = watch("theme");
  const showAuditLog = watch("show_audit_log");
  const autoScanRepos = watch("auto_scan_repos");
  const launchAtStartup = watch("launch_at_startup");
  const startMinimized = watch("start_minimized");
  const closeToTray = watch("close_to_tray");

  // Apply theme immediately on select change (before Save)
  useEffect(() => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const resolved =
      selectedTheme === "Light"
        ? "light"
        : selectedTheme === "System"
          ? mql.matches
            ? "dark"
            : "light"
          : "dark";
    document.documentElement.dataset.theme = resolved;
  }, [selectedTheme]);

  const onSubmit = async (values: FormData) => {
    // Preserve the Smart Switching block, which is managed by its own toggles.
    const base: AppSettings = data ?? {
      theme: "Dark",
      show_audit_log: true,
      auto_scan_repos: false,
      smart_switching: DEFAULT_SMART,
      onboarded: false,
      launch_at_startup: false,
      start_minimized: false,
      close_to_tray: false,
    };
    await update({ ...base, ...values });
    toast("Settings saved", "success");
  };

  if (loading && !data) return <LoadingState />;
  if (error && !data) return <ErrorState error={error} onRetry={fetch} />;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-6 max-w-lg">
      {/* Appearance */}
      <section>
        <SectionHeader title="Appearance" description="Customize how GitPersona looks." />
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) px-4 overflow-hidden">
          <SettingRow label="Theme" description="Choose your preferred color scheme.">
            <Controller
              control={control}
              name="theme"
              render={({ field }) => (
                <Select items={THEME_OPTIONS} value={field.value} onValueChange={field.onChange} />
              )}
            />
          </SettingRow>
          <SettingRow label="Language" description="Interface language.">
            <Select items={[{ label: "English", value: "en" }]} value="en" disabled />
          </SettingRow>
        </div>
      </section>

      <Separator />

      {/* Startup */}
      <section>
        <SectionHeader title="Startup" description="Control GitPersona's behavior on launch." />
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) px-4 overflow-hidden">
          <SettingRow
            label="Auto-scan repositories"
            description="Detect and import Git repositories on startup."
          >
            <Switch
              checked={autoScanRepos}
              onCheckedChange={(v) => setValue("auto_scan_repos", v)}
            />
          </SettingRow>
          <SettingRow label="Launch at login" description="Start GitPersona when you log in.">
            <Switch
              checked={launchAtStartup}
              onCheckedChange={(v) => setValue("launch_at_startup", v)}
            />
          </SettingRow>
          <SettingRow
            label="Start minimized"
            description="Open hidden in the system tray instead of the main window."
          >
            <Switch
              checked={startMinimized}
              onCheckedChange={(v) => setValue("start_minimized", v)}
            />
          </SettingRow>
          <SettingRow
            label="Keep running in tray"
            description="When you close the window, keep GitPersona running in the background so Smart Switching stays active."
          >
            <Switch checked={closeToTray} onCheckedChange={(v) => setValue("close_to_tray", v)} />
          </SettingRow>
        </div>
      </section>

      <Separator />

      {/* Smart Switching */}
      <section>
        <SectionHeader
          title="Smart Switching"
          description="Automatically apply the assigned identity when you work in a repository."
        />
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) px-4 overflow-hidden">
          <SettingRow
            label="Enable Smart Switching"
            description="Watch tracked repositories and switch identity on Git activity."
          >
            <Switch checked={smart.enabled} onCheckedChange={(v) => setSmartEnabled(v)} />
          </SettingRow>
          <SettingRow
            label="Start watching on launch"
            description="Resume watching automatically when GitPersona opens."
          >
            <Switch
              checked={smart.start_on_launch}
              onCheckedChange={(v) => patchSmart({ start_on_launch: v })}
            />
          </SettingRow>
          <SettingRow
            label="Confirm before switch"
            description="Ask for confirmation before applying an automatic switch."
          >
            <Switch
              checked={smart.confirm_before_switch}
              onCheckedChange={(v) => patchSmart({ confirm_before_switch: v })}
            />
          </SettingRow>
          <SettingRow
            label="Show notification"
            description="Show a toast when an identity is switched automatically."
          >
            <Switch
              checked={smart.show_notification}
              onCheckedChange={(v) => patchSmart({ show_notification: v })}
            />
          </SettingRow>
          <SettingRow
            label="Auto SSH"
            description="Follow the profile's SSH identity via the managed ~/.ssh/config."
          >
            <Switch checked={smart.auto_ssh} onCheckedChange={(v) => patchSmart({ auto_ssh: v })} />
          </SettingRow>
          <SettingRow
            label="Auto credential"
            description="Follow the profile's HTTPS credential in the Windows Credential Manager."
          >
            <Switch
              checked={smart.auto_credential}
              onCheckedChange={(v) => patchSmart({ auto_credential: v })}
            />
          </SettingRow>
        </div>
      </section>

      <Separator />

      {/* Activity */}
      <section>
        <SectionHeader title="Activity & Notifications" />
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) px-4 overflow-hidden">
          <SettingRow
            label="Show audit log"
            description="Record identity changes in the Activity view."
          >
            <Switch checked={showAuditLog} onCheckedChange={(v) => setValue("show_audit_log", v)} />
          </SettingRow>
          <SettingRow
            label="Desktop notifications"
            description="Notify when profiles are switched."
          >
            <Switch checked={false} onCheckedChange={() => {}} disabled />
          </SettingRow>
        </div>
      </section>

      <Separator />

      {/* Experimental */}
      <section>
        <SectionHeader title="Experimental" description="Features that are still in development." />
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) px-4 overflow-hidden">
          <SettingRow
            label="Repository auto-detect"
            description="Automatically assign profiles based on remote URLs."
          >
            <div className="flex items-center gap-2">
              <Badge variant="warning">Beta</Badge>
              <Switch checked={false} onCheckedChange={() => {}} disabled />
            </div>
          </SettingRow>
          <SettingRow
            label="GPG auto-configure"
            description="Automatically wire signing keys when switching profiles."
          >
            <div className="flex items-center gap-2">
              <Badge variant="warning">Beta</Badge>
              <Switch checked={false} onCheckedChange={() => {}} disabled />
            </div>
          </SettingRow>
        </div>
      </section>

      <Separator />

      {/* Version */}
      <section>
        <SectionHeader title="Version" />
        <div className="rounded-(--radius-xl) bg-(--color-surface) border border-(--color-border) px-4 overflow-hidden">
          <SettingRow label="GitPersona" description="Developer Identity Manager">
            <Badge variant="default">v{version}</Badge>
          </SettingRow>
          <SettingRow label="Runtime" description="Tauri 2 on Windows 11">
            <Badge variant="default">Stable</Badge>
          </SettingRow>
        </div>
      </section>

      <div>
        <Button type="submit" variant="ghost" disabled={loading}>
          {loading ? "Saving…" : "Save Settings"}
        </Button>
      </div>
    </form>
  );
}
