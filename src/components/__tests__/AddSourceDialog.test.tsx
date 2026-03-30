import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { AddSourceDialog } from "../AddSourceDialog";
import { mockIPC } from "../../test/setup";

describe("AddSourceDialog", () => {
  const defaultProps = {
    onClose: vi.fn(),
    onAdded: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the dialog title", () => {
    render(<AddSourceDialog {...defaultProps} />);
    // The dialog renders addSourceTitle — "新增來源" in zh-TW
    expect(document.querySelector(".dialog-box")).toBeTruthy();
  });

  it("renders name input field", () => {
    render(<AddSourceDialog {...defaultProps} />);
    const nameInput = document.querySelector("input[placeholder='company-skills']");
    expect(nameInput).toBeTruthy();
  });

  it("renders repository URL input field", () => {
    render(<AddSourceDialog {...defaultProps} />);
    const urlInput = document.querySelector("input[placeholder='https://gitlab.com/...']");
    expect(urlInput).toBeTruthy();
  });

  it("renders personal access token input field for gitlab", () => {
    render(<AddSourceDialog {...defaultProps} />);
    const tokenInput = document.querySelector("input[placeholder='glpat-...']");
    expect(tokenInput).toBeTruthy();
  });

  it("add button is disabled when fields are empty", () => {
    render(<AddSourceDialog {...defaultProps} />);
    const addBtn = document.querySelector(".btn-primary") as HTMLButtonElement;
    expect(addBtn).toBeTruthy();
    expect(addBtn.disabled).toBe(true);
  });

  it("add button is enabled when all required fields are filled", () => {
    render(<AddSourceDialog {...defaultProps} />);
    const nameInput = document.querySelector("input[placeholder='company-skills']") as HTMLInputElement;
    const urlInput = document.querySelector("input[placeholder='https://gitlab.com/...']") as HTMLInputElement;
    const tokenInput = document.querySelector("input[placeholder='glpat-...']") as HTMLInputElement;

    fireEvent.change(nameInput, { target: { value: "My Source" } });
    fireEvent.change(urlInput, { target: { value: "https://gitlab.com/test/repo" } });
    fireEvent.change(tokenInput, { target: { value: "glpat-abc123" } });

    const addBtn = document.querySelector(".btn-primary") as HTMLButtonElement;
    expect(addBtn.disabled).toBe(false);
  });

  it("calls onClose when cancel button is clicked", () => {
    render(<AddSourceDialog {...defaultProps} />);
    const cancelBtn = document.querySelector(".btn-secondary") as HTMLButtonElement;
    fireEvent.click(cancelBtn);
    expect(defaultProps.onClose).toHaveBeenCalled();
  });

  it("shows error when add_source invoke rejects", async () => {
    render(<AddSourceDialog {...defaultProps} />);
    const nameInput = document.querySelector("input[placeholder='company-skills']") as HTMLInputElement;
    const urlInput = document.querySelector("input[placeholder='https://gitlab.com/...']") as HTMLInputElement;
    const tokenInput = document.querySelector("input[placeholder='glpat-...']") as HTMLInputElement;

    fireEvent.change(nameInput, { target: { value: "My Source" } });
    fireEvent.change(urlInput, { target: { value: "https://gitlab.com/test/repo" } });
    fireEvent.change(tokenInput, { target: { value: "glpat-abc123" } });

    mockIPC("add_source", () => {
      throw { kind: "Remote", message: "401 Unauthorized" };
    });

    const addBtn = document.querySelector(".btn-primary") as HTMLButtonElement;
    fireEvent.click(addBtn);

    await waitFor(() => {
      expect(document.querySelector(".mcp-error")).toBeTruthy();
    });
  });

  it("calls onAdded and onClose after successful add", async () => {
    render(<AddSourceDialog {...defaultProps} />);
    const nameInput = document.querySelector("input[placeholder='company-skills']") as HTMLInputElement;
    const urlInput = document.querySelector("input[placeholder='https://gitlab.com/...']") as HTMLInputElement;
    const tokenInput = document.querySelector("input[placeholder='glpat-...']") as HTMLInputElement;

    fireEvent.change(nameInput, { target: { value: "My Source" } });
    fireEvent.change(urlInput, { target: { value: "https://gitlab.com/test/repo" } });
    fireEvent.change(tokenInput, { target: { value: "glpat-abc123" } });

    mockIPC("add_source", () => ({
      id: "uuid-123",
      name: "My Source",
      type: "gitlab",
      url: "https://gitlab.com/test/repo",
      folder_id: null,
      added_at: Date.now(),
    }));

    const addBtn = document.querySelector(".btn-primary") as HTMLButtonElement;
    fireEvent.click(addBtn);

    await waitFor(() => {
      expect(defaultProps.onAdded).toHaveBeenCalled();
      expect(defaultProps.onClose).toHaveBeenCalled();
    });
  });
});
