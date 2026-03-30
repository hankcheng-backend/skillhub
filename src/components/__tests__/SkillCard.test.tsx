import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { SkillCard } from "../SkillCard";
import type { Skill } from "../../types";

const mockSkill: Skill = {
  id: "claude:test-skill",
  folder_name: "test-skill",
  name: "Test Skill",
  description: "A test skill description",
  origin_agent: "claude",
  tags: null,
  notes: null,
  discovered_at: null,
  updated_at: null,
  synced_to: [],
};

const defaultProps = {
  skill: mockSkill,
  enabledAgents: ["claude"],
  onOpen: vi.fn(),
  onSync: vi.fn(),
};

describe("SkillCard", () => {
  it("renders skill name", () => {
    render(<SkillCard {...defaultProps} />);
    expect(screen.getByText("Test Skill")).toBeInTheDocument();
  });

  it("renders skill description", () => {
    render(<SkillCard {...defaultProps} />);
    expect(screen.getByText("A test skill description")).toBeInTheDocument();
  });

  it("shows folder_name when name is null", () => {
    const noNameSkill: Skill = { ...mockSkill, name: null };
    render(<SkillCard {...defaultProps} skill={noNameSkill} />);
    expect(screen.getByText("test-skill")).toBeInTheDocument();
  });

  it("shows noDescription text when description is null", () => {
    const noDescSkill: Skill = { ...mockSkill, description: null };
    render(<SkillCard {...defaultProps} skill={noDescSkill} />);
    // The component renders t("noDescription") which is "無說明" in zh-TW
    const descEl = document.querySelector(".skill-card-desc");
    expect(descEl).toBeTruthy();
    expect(descEl?.textContent?.length).toBeGreaterThan(0);
  });

  it("renders the skill card container", () => {
    render(<SkillCard {...defaultProps} />);
    expect(document.querySelector(".skill-card")).toBeTruthy();
  });

  it("renders agent badge row for enabled agents", () => {
    render(<SkillCard {...defaultProps} />);
    expect(document.querySelector(".agent-badge-row")).toBeTruthy();
  });

  it("renders origin agent badge for claude when claude is enabled", () => {
    render(<SkillCard {...defaultProps} />);
    const originBadge = document.querySelector(".agent-badge--origin");
    expect(originBadge).toBeTruthy();
  });

  it("shows no synced-to badges when synced_to is empty", () => {
    render(<SkillCard {...defaultProps} />);
    const syncedBadges = document.querySelectorAll(".agent-badge--clickable");
    // No clickable badges when synced_to is empty (origin agent is not clickable)
    expect(syncedBadges.length).toBe(0);
  });

  it("renders skill card header", () => {
    render(<SkillCard {...defaultProps} />);
    expect(document.querySelector(".skill-card-header")).toBeTruthy();
  });
});
