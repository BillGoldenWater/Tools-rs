import { Panel } from "../../component/Panel";
import { jump_to, type PageId } from "../page";

export function Entry() {
  return (
    <div class={"p-8"}>
      <Panel class={"flex flex-wrap items-center gap-8 p-8"}>
        <EntryItem
          id={"navigation_compass_solver"}
          name={"引航罗盘推演"}
          iconUrl={"/assets/img/function_icon/navigation_compass_solver.webp"}
        />
        <EntryItem
          id={"aeroplane_chess_solver"}
          name={"飞行棋推演"}
          iconUrl={"/assets/img/function_icon/aeroplane_chess_solver.webp"}
          by={`\
"Fei xing qi board (RYBG).svg" \
by [Mliu92](https://commons.wikimedia.org/wiki/User:Mliu92)
Source: https://commons.wikimedia.org/wiki/File:Fei_xing_qi_board_(RYBG).svg
License: [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
Rendered to Webp\
`}
        />
      </Panel>
    </div>
  );
}

function EntryItem(props: { id: PageId; name: string; iconUrl: string, by?: string }) {
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
        title={props.by}
      />
      <div class={"flex justify-center p-2"}>{props.name}</div>
    </div>
  );
}
