import { describe, expect, it } from "vitest";
import { describeVerifyFailure, validateApplicationId } from "./validation";

const VALID_ID = "0123456789abcdef0123456789abcdef";

describe("validateApplicationId", () => {
  it("accepts 32 lowercase hexadecimal characters", () => {
    expect(validateApplicationId(VALID_ID)).toEqual({ ok: true, value: VALID_ID });
  });

  it("trims surrounding whitespace and lowercases before checking", () => {
    expect(validateApplicationId(`  ${VALID_ID.toUpperCase()}\n`)).toEqual({ ok: true, value: VALID_ID });
  });

  it.each(["", "   ", "\t\n"])("reports an empty value for %j", (input) => {
    expect(validateApplicationId(input)).toMatchObject({ ok: false, problem: "empty" });
  });

  it("rejects characters outside 0 to 9 and a to f", () => {
    const result = validateApplicationId(`${VALID_ID.slice(0, 31)}g`);
    expect(result).toMatchObject({ ok: false, problem: "characters" });
    expect(result.ok ? "" : result.message).toMatch(/0 to 9.*a to f/);
  });

  it("treats whitespace inside the value as a character problem", () => {
    const spaced = `${VALID_ID.slice(0, 16)} ${VALID_ID.slice(16)}`;
    expect(validateApplicationId(spaced)).toMatchObject({ ok: false, problem: "characters" });
  });

  it.each([
    [VALID_ID.slice(0, 31), 31],
    [`${VALID_ID}a`, 33],
  ])("rejects %s for its length", (input, length) => {
    const result = validateApplicationId(input);
    expect(result).toMatchObject({ ok: false, problem: "length" });
    expect(result.ok ? "" : result.message).toContain(`32 characters long; this one has ${length}`);
  });
});

describe("describeVerifyFailure", () => {
  it.each([
    ["the Wargaming application ID was rejected", "rejected"],
    ["Wargaming is rate limiting requests; try again shortly", "rateLimited"],
    ["could not reach Wargaming: operation timed out", "network"],
    ["unexpected response from Wargaming: missing data", "unexpected"],
    ["Wargaming returned an error: 504 SOURCE_NOT_AVAILABLE", "unexpected"],
  ])("classifies %j", (reason, kind) => {
    expect(describeVerifyFailure(reason).kind).toBe(kind);
  });

  it("never repeats the backend reason, which can carry the application ID", () => {
    const reason = `could not reach Wargaming: error sending request for url (https://api.worldofwarships.eu/wows/encyclopedia/info/?application_id=${VALID_ID})`;
    const failure = describeVerifyFailure(new Error(reason));
    expect(failure.kind).toBe("network");
    expect(failure.message).not.toContain(VALID_ID);
  });
});
