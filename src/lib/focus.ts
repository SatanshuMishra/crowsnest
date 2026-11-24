export type HeadingFocus = (heading: HTMLElement) => void;

export function headingFocus(): HeadingFocus {
  let viewShown = false;
  return (heading) => {
    if (viewShown) {
      heading.focus();
    }
    viewShown = true;
  };
}

export const focusViewHeading = headingFocus();
