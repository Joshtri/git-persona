import { z } from "zod";
import { CREDENTIAL_HOST_VALUES } from "@/lib/credential-hosts";

export const identitySchema = z.object({
  name: z.string().min(1, "Name is required").max(100),
  email: z.string().email("Invalid email address"),
  signing_key: z.string().optional(),
});

export const profileSchema = z.object({
  label: z.string().min(1, "Label is required").max(50),
  identity: identitySchema,
  color: z.string().optional(),
});

export type IdentityFormData = z.infer<typeof identitySchema>;
export type ProfileFormData = z.infer<typeof profileSchema>;

export const sshImportSchema = z.object({
  privateKeyPath: z.string().min(1, "Choose a private key file"),
  hostAlias: z.string().max(100).optional(),
  hostName: z.string().max(255).optional(),
});

export const sshGenerateSchema = z.object({
  label: z.string().min(1, "Label is required").max(50),
  algorithm: z.enum(["Ed25519", "Rsa"]),
  comment: z.string().max(255).optional(),
  outputDir: z.string().min(1, "Choose an output folder"),
  fileName: z
    .string()
    .min(1, "File name is required")
    .max(80)
    .regex(/^[A-Za-z0-9._-]+$/, "Use letters, numbers, dots, dashes or underscores"),
  hostAlias: z.string().max(100).optional(),
  hostName: z.string().max(255).optional(),
});

export type SshImportFormData = z.infer<typeof sshImportSchema>;
export type SshGenerateFormData = z.infer<typeof sshGenerateSchema>;

export const credentialSchema = z.object({
  host: z.enum(CREDENTIAL_HOST_VALUES, { message: "Choose a supported host" }),
  username: z.string().min(1, "Username is required").max(100),
  token: z.string().min(1, "Token is required").max(500),
  // "" means unassigned; any other value is a profile id.
  profileId: z.string().optional(),
});

export const credentialEditSchema = z.object({
  username: z.string().min(1, "Username is required").max(100),
  // Optional rotation — leave blank to keep the existing token.
  token: z.string().max(500).optional(),
});

export type CredentialFormData = z.infer<typeof credentialSchema>;
export type CredentialEditFormData = z.infer<typeof credentialEditSchema>;
