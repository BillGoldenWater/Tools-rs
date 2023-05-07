import { gen_path } from "../page";

export function Entry() {
  return (
    <div class={"flex flex-col items-center"}>
      <div>
        <a class={"underline"} href={gen_path("navigation_compass_solver")}>
          引航罗盘推演
        </a>
      </div>
    </div>
  );
}
