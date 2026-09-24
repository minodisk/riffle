// Which of the left pane, the filmstrip and the right pane are shown.
export type Panels = { left: boolean; strip: boolean; right: boolean };

export type Panel = keyof Panels;

export function toggle(panels: Panels, which: Panel): Panels {
  return { ...panels, [which]: !panels[which] };
}

// Lightroom's Tab: hide both side panes when either is shown, else show both.
export function toggleSides(panels: Panels): Panels {
  const shown = !(panels.left || panels.right);
  return { ...panels, left: shown, right: shown };
}
