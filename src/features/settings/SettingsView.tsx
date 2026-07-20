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
// import { Badge, Button, Separator, Switch } from "@/components/ui";
import type { AppSettings, Theme } from "@/ipc/types.gen";
import { useFeedbackStore } from "@/stores/feedback";
import { useSettingsStore } from "@/stores/settings";
import { Select, type SelectOption } from "../inputs/non-form/select";
import { SectionHeader } from "./components/Section.Header";
import { SettingRow } from "./components/Section.SettingRow";

const schema = z.object({
  theme: z.enum(["Dark", "Light", "System"]),
  show_audit_log: z.boolean(),
  auto_scan_repos: z.boolean(),
});

type FormData = z.infer<typeof schema>;
const THEMES: Theme[] = ["Dark", "Light", "System"];

const THEME_OPTIONS: SelectOption[] = THEMES.map((theme) => ({
  label: theme,
  value: theme,
}));

export function SettingsView() {
  const { data, loading, error, fetch, update } = useSettingsStore();
  const toast = useFeedbackStore((s) => s.toast);

  useEffect(() => {
    fetch();
  }, [fetch]);

  const { control, handleSubmit, reset, watch, setValue } = useForm<FormData>({
    resolver: zodResolver(schema),
    defaultValues: data ?? {
      theme: "Dark",
      show_audit_log: true,
      auto_scan_repos: false,
    },
  });

  useEffect(() => {
    if (data) reset(data);
  }, [data, reset]);

  const selectedTheme = watch("theme");
  const showAuditLog = watch("show_audit_log");
  const autoScanRepos = watch("auto_scan_repos");

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
    await update(values as AppSettings);
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
            <Switch checked={false} onCheckedChange={() => {}} disabled />
          </SettingRow>
          <SettingRow
            label="Start minimized"
            description="Open to system tray instead of main window."
          >
            <Switch checked={false} onCheckedChange={() => {}} disabled />
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
            <Badge variant="default">v0.1.0</Badge>
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
