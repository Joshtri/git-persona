export interface AppError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

export type ErrorCode =
  | "NOT_FOUND"
  | "VALIDATION"
  | "GIT_CONFIG"
  | "STORE"
  | "IO"
  | "CONFLICT"
  | "UNDO_UNAVAILABLE"
  | "NOT_IMPLEMENTED"
  | "INTERNAL";

export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value &&
    typeof (value as AppError).code === "string" &&
    typeof (value as AppError).message === "string"
  );
}

export function toAppError(value: unknown): AppError {
  if (isAppError(value)) return value;
  return { code: "INTERNAL", message: String(value) };
}
