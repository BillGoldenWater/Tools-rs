import { gen_path } from "../page";

export function Entry() {
  return (
    <div class={"flex flex-col items-center"}>
      <p>Tools:</p>
      <div>
        <a class={"underline"} href={gen_path("navigation_compass_solver")}>
          Navigation Compass Solver
        </a>
      </div>
    </div>
  );
}
