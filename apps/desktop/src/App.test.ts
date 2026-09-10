import { describe, expect, it } from "vitest";

describe("Phase 0 desktop contract", () => {
  it("keeps a stable product name", () => {
    expect("Helel").toBe("Helel");
  });
});
