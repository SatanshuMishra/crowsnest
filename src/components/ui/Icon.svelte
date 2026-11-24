<script lang="ts" module>
  export type IconName =
    | "settings"
    | "refresh"
    | "back"
    | "close"
    | "folder"
    | "check"
    | "warning"
    | "info"
    | "error"
    | "external"
    | "download"
    | "search"
    | "anchor"
    | "user"
    | "eye"
    | "eyeOff"
    | "palette"
    | "pin";

  const paths: Record<IconName, string> = {
    settings:
      "M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Zm7.4-2.1a7.6 7.6 0 0 0 0-2.8l2-1.6-2-3.4-2.4 1a7.4 7.4 0 0 0-2.4-1.4L14.3 2.6h-4l-.4 2.5a7.4 7.4 0 0 0-2.4 1.4l-2.4-1-2 3.4 2 1.6a7.6 7.6 0 0 0 0 2.8l-2 1.6 2 3.4 2.4-1a7.4 7.4 0 0 0 2.4 1.4l.4 2.5h4l.4-2.5a7.4 7.4 0 0 0 2.4-1.4l2.4 1 2-3.4-2-1.6Z",
    refresh: "M20 11a8 8 0 1 0-2.3 5.7M20 4v7h-7",
    back: "M15 18l-6-6 6-6",
    close: "M6 6l12 12M18 6L6 18",
    folder: "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z",
    check: "M5 12.5l4.5 4.5L19 7.5",
    warning: "M12 3l9.5 17h-19L12 3Zm0 6v5m0 3v.5",
    info: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Zm0-10v6m0-9.5V8",
    error: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Zm-3-12l6 6m0-6l-6 6",
    external: "M14 4h6v6m0-6L10 14M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5",
    download: "M12 4v11m0 0l-4.5-4.5M12 15l4.5-4.5M5 20h14",
    search: "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14Zm5-2l4 4",
    anchor: "M12 7a2 2 0 1 0 0-4 2 2 0 0 0 0 4Zm0 0v14m0 0a8 8 0 0 1-8-8m8 8a8 8 0 0 0 8-8M8 11h8",
    user: "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Zm-7 8a7 7 0 0 1 14 0",
    eye: "M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12Zm9.5 3a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z",
    eyeOff: "M3 3l18 18M10.6 6.1A9.8 9.8 0 0 1 12 6c5 0 8.5 4.5 9.5 6a17 17 0 0 1-2.7 3.3M6.6 6.6C4.4 8 3 10.2 2.5 12c1 1.5 4.5 6 9.5 6a9.6 9.6 0 0 0 4.4-1.1M9.9 9.9a3 3 0 0 0 4.2 4.2",
    palette:
      "M12 21a9 9 0 1 1 9-9c0 2.2-1.6 3.5-3.5 3.5h-1.8a1.9 1.9 0 0 0-1.3 3.2 1.4 1.4 0 0 1-1 2.3H12ZM7.5 11.5h.01M10 7.5h.01M14.5 7.5h.01M17 11h.01",
    pin: "M7.9 3.4h8.2M12 15v5.6M9.75 3.4v5.85L6.75 15h10.5l-3-5.75V3.4Z",
  };
</script>

<script lang="ts">
  interface Props {
    name: IconName;
    size?: number;
  }

  let { name, size = 16 }: Props = $props();
</script>

<svg
  class="icon"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.75"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  <path d={paths[name]} />
</svg>

<style>
  .icon {
    flex: none;
    display: block;
  }
</style>
