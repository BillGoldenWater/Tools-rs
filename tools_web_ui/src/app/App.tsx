import { Dynamic } from "solid-js/web";
import { PageId, pages } from "./page";
import { NotFound } from "./NotFound";

export function App() {
  const search = new URLSearchParams(window.location.search);
  const pageId: PageId = (search.get("page") || "") as PageId;

  return <Dynamic component={pages[pageId] || NotFound} />;
}
