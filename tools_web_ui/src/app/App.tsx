import { Dynamic } from "solid-js/web";
import { PageId, pages } from "./page";
import { NotFound } from "./NotFound";
import { onCleanup, onMount } from "solid-js";

export function App() {
  const search = new URLSearchParams(window.location.search);
  const pageId: PageId = (search.get("page") || "") as PageId;

  onMount(() => {
    function onKeyUp(e: KeyboardEvent) {
      if (e.key === "Escape") {
        window.history.back();
      }
    }

    window.addEventListener("keyup", onKeyUp);

    onCleanup(() => {
      window.removeEventListener("keyup", onKeyUp);
    });
  });

  return <Dynamic component={pages[pageId] || NotFound} />;
}
