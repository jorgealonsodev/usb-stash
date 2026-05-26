import { describe, it, expect } from "vitest";
import zxcvbn from "zxcvbn";

describe("canSubmit computed logic", () => {
  function evaluateCanSubmit(password: string, confirm: string): boolean {
    const passwordValid = password.length >= 12;
    const entropy = zxcvbn(password).score;
    const passwordsMatch = password === confirm;
    return passwordValid && entropy >= 3 && passwordsMatch;
  }

  it("rejects password shorter than 12 chars", () => {
    expect(evaluateCanSubmit("short", "short")).toBe(false);
  });

  it("rejects weak password (zxcvbn < 3) even if 12+ chars", () => {
    // "aaaaaaaaaaaa" is 12 chars but very weak
    expect(evaluateCanSubmit("aaaaaaaaaaaa", "aaaaaaaaaaaa")).toBe(false);
  });

  it("rejects when passwords don't match", () => {
    // "CorrectHorse!1" should be strong enough
    expect(evaluateCanSubmit("CorrectHorse!1", "CorrectHorse!2")).toBe(false);
  });

  it("accepts valid password with matching confirm", () => {
    const pw = "CorrectHorse!1";
    expect(evaluateCanSubmit(pw, pw)).toBe(true);
  });

  it("requires all three gates: length, entropy, match", () => {
    const pw = "MyS3cur3P@ss!";
    expect(evaluateCanSubmit(pw, pw)).toBe(true);
    expect(evaluateCanSubmit(pw.slice(0, 8), pw.slice(0, 8))).toBe(false);
    expect(evaluateCanSubmit(pw, pw + "x")).toBe(false);
  });
});
