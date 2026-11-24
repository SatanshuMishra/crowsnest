import { describe, expect, it } from "vitest";
import { accountIdOf, playerLinks, wowsKarmaUrl, wowsNumbersUrl } from "./links";

describe("wowsNumbersUrl", () => {
  it("uses the bare domain for EU and a regional subdomain elsewhere", () => {
    expect(wowsNumbersUrl("eu", 500123, "Captain_One")).toBe("https://wows-numbers.com/player/500123,Captain_One/");
    expect(wowsNumbersUrl("na", 1001, "Skipper")).toBe("https://na.wows-numbers.com/player/1001,Skipper/");
    expect(wowsNumbersUrl("asia", 2002, "Mate")).toBe("https://asia.wows-numbers.com/player/2002,Mate/");
  });

  it("encodes characters that are not safe in a path", () => {
    expect(wowsNumbersUrl("eu", 7, "a b")).toBe("https://wows-numbers.com/player/7,a%20b/");
  });
});

describe("wowsKarmaUrl", () => {
  it("follows the same subdomain rule", () => {
    expect(wowsKarmaUrl("eu", 500123, "Captain_One")).toBe("https://wows-karma.com/player/500123,Captain_One/");
    expect(wowsKarmaUrl("na", 1001, "Skipper")).toBe("https://na.wows-karma.com/player/1001,Skipper/");
    expect(wowsKarmaUrl("asia", 2002, "Mate")).toBe("https://asia.wows-karma.com/player/2002,Mate/");
  });
});

describe("playerLinks", () => {
  it("builds both links for players with an account", () => {
    const links = playerLinks("na", { kind: "hidden", accountId: 42 }, "Quiet");
    expect(links.map((link) => link.url)).toEqual([
      "https://na.wows-numbers.com/player/42,Quiet/",
      "https://na.wows-karma.com/player/42,Quiet/",
    ]);
  });

  it("has no links without a region or an account", () => {
    expect(playerLinks(null, { kind: "hidden", accountId: 42 }, "Quiet")).toEqual([]);
    expect(playerLinks("eu", { kind: "bot" }, ":Bot:")).toEqual([]);
    expect(playerLinks("eu", { kind: "notFound" }, "Ghost")).toEqual([]);
  });

  it("reads the account id from loaded and hidden stats only", () => {
    expect(accountIdOf({ kind: "hidden", accountId: 5 })).toBe(5);
    expect(accountIdOf({ kind: "failed", reason: "timeout" })).toBeNull();
  });
});
