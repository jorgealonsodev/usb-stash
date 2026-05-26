import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import EntropyBar from "../components/EntropyBar.svelte";

describe("EntropyBar", () => {
  const cases = [
    { score: 0, label: "Débil" },
    { score: 1, label: "Regular" },
    { score: 2, label: "Buena" },
    { score: 3, label: "Fuerte" },
    { score: 4, label: "Excelente" },
  ];

  for (const { score, label } of cases) {
    it(`renders label "${label}" for score ${score}`, () => {
      render(EntropyBar, { props: { score } });
      expect(screen.getByText(label)).toBeTruthy();
    });

    it(`sets aria-valuenow to ${score}`, () => {
      const { container } = render(EntropyBar, { props: { score } });
      const bar = container.querySelector('[role="progressbar"]');
      expect(bar?.getAttribute("aria-valuenow")).toBe(String(score));
    });
  }

  it("clamps score to valid range", () => {
    const { container } = render(EntropyBar, { props: { score: 10 } });
    const bar = container.querySelector('[role="progressbar"]');
    expect(bar?.getAttribute("aria-valuenow")).toBe("4");
  });
});
