import { describe, it, expect } from "vitest";
import { render, fireEvent } from "@testing-library/svelte";
import PasswordInput from "../components/PasswordInput.svelte";

describe("PasswordInput", () => {
  it("renders with password field visible by default", () => {
    const { container } = render(PasswordInput, {
      props: { value: "", id: "pw" },
    });
    const inputs = container.querySelectorAll("input");
    // Password input (type="password") should be visible, text input hidden
    const pwInput = inputs[0] as HTMLInputElement;
    const textInput = inputs[1] as HTMLInputElement;
    expect(pwInput.type).toBe("password");
    expect(pwInput.classList.contains("hidden")).toBe(false);
    expect(textInput.classList.contains("hidden")).toBe(true);
  });

  it("shows text input and hides password input when toggled", async () => {
    const { container } = render(PasswordInput, {
      props: { value: "", id: "pw" },
    });
    const toggle = container.querySelector("button.toggle")!;

    await fireEvent.click(toggle);

    const inputs = container.querySelectorAll("input");
    const pwInput = inputs[0] as HTMLInputElement;
    const textInput = inputs[1] as HTMLInputElement;
    expect(pwInput.classList.contains("hidden")).toBe(true);
    expect(textInput.classList.contains("hidden")).toBe(false);
  });

  it("toggles back to password on second click", async () => {
    const { container } = render(PasswordInput, {
      props: { value: "", id: "pw" },
    });
    const toggle = container.querySelector("button.toggle")!;

    await fireEvent.click(toggle);
    await fireEvent.click(toggle);

    const pwInput = container.querySelectorAll("input")[0] as HTMLInputElement;
    expect(pwInput.classList.contains("hidden")).toBe(false);
  });

  it("respects disabled prop", () => {
    const { container } = render(PasswordInput, {
      props: { value: "", disabled: true },
    });
    const inputs = container.querySelectorAll("input");
    const toggle = container.querySelector("button.toggle");
    expect(inputs[0].hasAttribute("disabled")).toBe(true);
    expect(toggle?.hasAttribute("disabled")).toBe(true);
  });
});
