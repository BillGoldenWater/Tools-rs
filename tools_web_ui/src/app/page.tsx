import { type Component, lazy } from "solid-js";
import { importAsDefault } from "../utils/importAsDefault";
import { AeroplaneChessSolver } from "./page/AeroplaneChessSolver";
import { NavigationCompassSolver } from "./page/NavigationCompassSolver";

export const pages = {
  "": lazy(() => importAsDefault<Component>(import("./page/Entry"), "Entry")),
  navigation_compass_solver: NavigationCompassSolver,
  aeroplane_chess_solver: AeroplaneChessSolver,
};

export type PageId = keyof typeof pages;

export function gen_path(page: PageId): string {
  return `?page=${page}`;
}

export function jump_to(page: PageId): () => void {
  return () => {
    window.location.href = gen_path(page);
  };
}
