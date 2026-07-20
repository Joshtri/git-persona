/// Supported HTTPS credential hosts, mirrored from the Rust `SUPPORTED_HOSTS`
/// allow-list. The backend is authoritative — this list drives the create form
/// and provider grouping only.
export const CREDENTIAL_HOST_VALUES = [
  "github.com",
  "gitlab.com",
  "bitbucket.org",
  "dev.azure.com",
] as const;

export type CredentialHost = (typeof CREDENTIAL_HOST_VALUES)[number];

const PROVIDER_LABELS: Record<CredentialHost, string> = {
  "github.com": "GitHub",
  "gitlab.com": "GitLab",
  "bitbucket.org": "Bitbucket",
  "dev.azure.com": "Azure DevOps",
};

/// Human-facing provider name for a host, falling back to the raw host.
export function providerLabel(host: string): string {
  return PROVIDER_LABELS[host as CredentialHost] ?? host;
}

export const HOST_SELECT_OPTIONS = CREDENTIAL_HOST_VALUES.map((host) => ({
  value: host,
  label: `${PROVIDER_LABELS[host]} · ${host}`,
}));

export interface HostGuidance {
  /// The field label a provider expects (PAT vs App Password).
  tokenLabel: string;
  /// Short note explaining what secret to paste; "" for unknown hosts.
  note: string;
}

const HOST_GUIDANCE: Record<CredentialHost, HostGuidance> = {
  "github.com": {
    tokenLabel: "Personal Access Token",
    note: "GitHub no longer supports account passwords for Git over HTTPS. Use a Personal Access Token (PAT) instead.",
  },
  "gitlab.com": {
    tokenLabel: "Personal Access Token",
    note: "Use a GitLab Personal Access Token with the write_repository scope — not your account password.",
  },
  "bitbucket.org": {
    tokenLabel: "App Password",
    note: "Bitbucket requires an App Password for Git over HTTPS — not your account password.",
  },
  "dev.azure.com": {
    tokenLabel: "Personal Access Token",
    note: "Use an Azure DevOps Personal Access Token (PAT) with Code (read/write) scope.",
  },
};

/// Provider-specific token labelling and guidance for the credential form.
export function hostGuidance(host: string): HostGuidance {
  return HOST_GUIDANCE[host as CredentialHost] ?? { tokenLabel: "Token / Password", note: "" };
}
