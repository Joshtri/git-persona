import { useBootstrapStore } from "@/stores/bootstrap";

/**
 * Returns whether a server-controlled feature flag is enabled.
 * Defaults to `false` until the bootstrap data is loaded.
 *
 * @example
 * const enabled = useFeatureFlag("ai-commit-message");
 */
export function useFeatureFlag(key: string): boolean {
  return useBootstrapStore((s) => s.featureFlags[key] ?? false);
}
