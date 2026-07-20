import type { AuditEntry, Profile } from "@/ipc/types.gen";

export const MOCK_PROFILES: Profile[] = [
  {
    id: "p1",
    label: "Work",
    identity: { name: "Alex Chen", email: "alex.chen@company.com", signing_key: "4A3B2C1D" },
    color: "#6366f1",
  },
  {
    id: "p2",
    label: "Personal",
    identity: { name: "Alex Chen", email: "alex@gmail.com" },
    color: "#10b981",
  },
  {
    id: "p3",
    label: "Open Source",
    identity: { name: "achenblog", email: "alex@users.noreply.github.com" },
    color: "#f59e0b",
  },
  {
    id: "p4",
    label: "Client — Acme",
    identity: { name: "Alex C.", email: "alex@acme-client.com", signing_key: "9F8E7D6C" },
    color: "#ef4444",
  },
];

export const ACTIVE_PROFILE = MOCK_PROFILES[0];

export const MOCK_ACTIVITY: AuditEntry[] = [
  {
    id: "a1",
    timestamp: new Date(Date.now() - 1000 * 60 * 5).toISOString(),
    action: "profile.switch",
    profile_id: "p1",
    repo_path: "/home/alex/projects/api-service",
  },
  {
    id: "a2",
    timestamp: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
    action: "profile.switch",
    profile_id: "p2",
    repo_path: "/home/alex/projects/personal-site",
  },
  {
    id: "a3",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString(),
    action: "identity.apply",
    profile_id: "p1",
    repo_path: undefined,
  },
  {
    id: "a4",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 5).toISOString(),
    action: "profile.create",
    profile_id: "p4",
    repo_path: undefined,
  },
  {
    id: "a5",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(),
    action: "profile.switch",
    profile_id: "p3",
    repo_path: "/home/alex/projects/oss-lib",
  },
  {
    id: "a6",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 26).toISOString(),
    action: "identity.apply",
    profile_id: "p2",
    repo_path: undefined,
  },
  {
    id: "a7",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 48).toISOString(),
    action: "profile.switch",
    profile_id: "p1",
    repo_path: "/home/alex/projects/acme-dashboard",
  },
  {
    id: "a8",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 50).toISOString(),
    action: "profile.create",
    profile_id: "p3",
    repo_path: undefined,
  },
];

export interface MockSshKey {
  id: string;
  name: string;
  fingerprint: string;
  type: "Ed25519" | "RSA-4096" | "ECDSA";
  created: string;
  profile_id?: string;
  comment: string;
}

export const MOCK_SSH_KEYS: MockSshKey[] = [
  {
    id: "k1",
    name: "Work MacBook",
    fingerprint: "SHA256:ABC123xyz789DEFghi456JKL",
    type: "Ed25519",
    created: "2024-01-15",
    profile_id: "p1",
    comment: "alex@work-macbook",
  },
  {
    id: "k2",
    name: "Personal Laptop",
    fingerprint: "SHA256:XYZ987uvw321STUvwx654MNO",
    type: "Ed25519",
    created: "2023-11-02",
    profile_id: "p2",
    comment: "alex@personal-laptop",
  },
  {
    id: "k3",
    name: "GitHub Deploy Key",
    fingerprint: "SHA256:PQR456stu123VWXyzb789ABC",
    type: "RSA-4096",
    created: "2023-08-20",
    comment: "deploy@acme",
  },
];

export interface MockCredential {
  id: string;
  name: string;
  host: string;
  username: string;
  type: "HTTPS" | "Token";
  created: string;
  profile_id?: string;
}

export const MOCK_CREDENTIALS: MockCredential[] = [
  {
    id: "c1",
    name: "GitHub (Work)",
    host: "github.com",
    username: "alex-work",
    type: "Token",
    created: "2024-02-01",
    profile_id: "p1",
  },
  {
    id: "c2",
    name: "GitLab",
    host: "gitlab.com",
    username: "achenblog",
    type: "Token",
    created: "2023-12-10",
    profile_id: "p3",
  },
];

export function getProfileById(id: string): Profile | undefined {
  return MOCK_PROFILES.find((p) => p.id === id);
}

export function getProfileColor(id: string | undefined): string {
  if (!id) return "#6366f1";
  return getProfileById(id)?.color ?? "#6366f1";
}

export function getProfileLabel(id: string | undefined): string {
  if (!id) return "Unassigned";
  return getProfileById(id)?.label ?? "Unknown";
}
