import { jump_to, PageId } from "../page";
import { Panel } from "../../component/Panel";

export function Entry() {
  return (
    <div class={"p-8"}>
      <Panel class={"flex flex-wrap items-center gap-8 p-8"}>
        <EntryItem
          id={"navigation_compass_solver"}
          name={"引航罗盘推演"}
          iconUrl={"/assets/img/function_icon/navigation_compass_solver.webp"}
        />
      </Panel>
    </div>
  );
}

function EntryItem(props: { id: PageId; name: string; iconUrl: string }) {
  return (
    <div
      onclick={jump_to(props.id)}
      class={
        "flex cursor-pointer select-none flex-col overflow-hidden rounded-xl border-2 border-solid border-gray-300 bg-gray-200 shadow-xl transition-all hover:scale-105 hover:shadow-2xl active:scale-100 dark:border-gray-500 dark:bg-gray-900"
      }
    >
      <img
        class={"block aspect-square h-32"}
        src={props.iconUrl}
        alt={"Icon"}
        draggable={false}
        elementtiming={""}
        fetchpriority={"high"}
      />
      <div class={"flex justify-center p-2"}>{props.name}</div>
    </div>
  );
}
