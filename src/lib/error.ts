import { t } from "./i18n";

type UnknownObject = Record<string, unknown>;

function asString(value: unknown): string | null {
  return typeof value === "string" && value.trim().length > 0 ? value : null;
}

export function extractErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message;
  }

  const direct = asString(error);
  if (direct) {
    return direct;
  }

  if (error && typeof error === "object") {
    const obj = error as UnknownObject;

    const candidates = [
      obj.error,
      obj.message,
      obj.msg,
      obj.reason,
      (obj.data as UnknownObject | undefined)?.error,
      (obj.data as UnknownObject | undefined)?.message,
      (obj.cause as UnknownObject | undefined)?.error,
      (obj.cause as UnknownObject | undefined)?.message,
    ];

    for (const candidate of candidates) {
      const text = asString(candidate);
      if (text) {
        return text;
      }
    }

    try {
      return JSON.stringify(obj);
    } catch {
      // fall through
    }
  }

  return String(error);
}

export function formatAddSourceError(error: unknown): string {
  const msg = extractErrorMessage(error);
  const lower = msg.toLowerCase();

  if (lower.includes("source name cannot be empty")) {
    return t("addSourceErrNameRequired");
  }
  if (lower.includes("gitlab source url is required")) {
    return t("addSourceErrRepoUrlRequired");
  }
  if (lower.includes("gitlab personal access token is required")) {
    return t("addSourceErrTokenRequired");
  }
  if (lower.includes("invalid repo url")) {
    return t("addSourceErrInvalidRepoUrl");
  }
  if (lower.includes("401 unauthorized") || lower.includes("forbidden")) {
    return t("addSourceErrUnauthorized");
  }
  if (
    lower.includes("gitlab api error") ||
    lower.includes("failed to send request") ||
    lower.includes("dns error") ||
    lower.includes("timed out")
  ) {
    return t("addSourceErrNetwork");
  }

  return msg;
}
