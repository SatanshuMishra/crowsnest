export const APPLICATION_ID_LENGTH = 32;

const HEX_ONLY = /^[0-9a-f]*$/;

export type ApplicationIdProblem = "empty" | "characters" | "length";

export type ApplicationIdCheck =
  | { ok: true; value: string }
  | { ok: false; problem: ApplicationIdProblem; message: string };

export function validateApplicationId(input: string): ApplicationIdCheck {
  const value = input.trim().toLowerCase();
  if (value.length === 0) {
    return { ok: false, problem: "empty", message: "Enter your application ID." };
  }
  if (!HEX_ONLY.test(value)) {
    return {
      ok: false,
      problem: "characters",
      message: "An application ID uses only the digits 0 to 9 and the letters a to f.",
    };
  }
  if (value.length !== APPLICATION_ID_LENGTH) {
    return {
      ok: false,
      problem: "length",
      message: `An application ID is ${APPLICATION_ID_LENGTH} characters long; this one has ${value.length}.`,
    };
  }
  return { ok: true, value };
}

export type VerifyFailureKind = "rejected" | "rateLimited" | "network" | "unexpected";

export interface VerifyFailure {
  kind: VerifyFailureKind;
  message: string;
}

const verifyFailureMessages: Record<VerifyFailureKind, string> = {
  rejected:
    "Wargaming did not accept this ID. Copy it again from your Standalone application on developers.wargaming.net.",
  rateLimited: "Wargaming is limiting requests right now. Wait a minute, then try again.",
  network: "Crowsnest could not reach Wargaming. Check your internet connection, then try again.",
  unexpected: "Wargaming sent an answer Crowsnest did not expect. Try again in a few minutes.",
};

function verifyFailureKind(reason: string): VerifyFailureKind {
  if (reason.includes("rejected")) {
    return "rejected";
  }
  if (reason.includes("rate limit")) {
    return "rateLimited";
  }
  if (reason.includes("could not reach")) {
    return "network";
  }
  return "unexpected";
}

export function describeVerifyFailure(error: unknown): VerifyFailure {
  const kind = verifyFailureKind(String(error).toLowerCase());
  return { kind, message: verifyFailureMessages[kind] };
}
