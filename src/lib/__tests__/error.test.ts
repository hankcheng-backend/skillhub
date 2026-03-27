import { describe, it, expect } from "vitest";
import { extractErrorMessage, formatAddSourceError } from "../error";

describe("extractErrorMessage", () => {
  it("extracts message from AppError-shaped object", () => {
    const err = { kind: "Db", message: "connection failed" };
    expect(extractErrorMessage(err)).toBe("connection failed");
  });

  it("extracts message from Error object", () => {
    expect(extractErrorMessage(new Error("test error"))).toBe("test error");
  });

  it("returns string directly", () => {
    expect(extractErrorMessage("plain string")).toBe("plain string");
  });

  it("returns fallback for null", () => {
    const result = extractErrorMessage(null);
    expect(typeof result).toBe("string");
    expect(result.length).toBeGreaterThan(0);
  });

  it("returns fallback for undefined", () => {
    const result = extractErrorMessage(undefined);
    expect(typeof result).toBe("string");
    expect(result.length).toBeGreaterThan(0);
  });

  it("extracts message from nested data object", () => {
    const err = { data: { message: "nested error" } };
    expect(extractErrorMessage(err)).toBe("nested error");
  });
});

describe("formatAddSourceError", () => {
  it("returns a string for AppError input", () => {
    const err = { kind: "Conflict", message: "Source name cannot be empty" };
    const result = formatAddSourceError(err);
    expect(typeof result).toBe("string");
    expect(result.length).toBeGreaterThan(0);
  });

  it("returns a string for network error", () => {
    const err = { kind: "Remote", message: "DNS error: failed to send request" };
    const result = formatAddSourceError(err);
    expect(typeof result).toBe("string");
    expect(result.length).toBeGreaterThan(0);
  });

  it("returns raw message for unknown errors", () => {
    const err = { kind: "Unknown", message: "some unknown error" };
    const result = formatAddSourceError(err);
    expect(result).toBe("some unknown error");
  });
});
