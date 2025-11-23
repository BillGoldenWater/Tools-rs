/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { wasm_init } from "tools_wasm";
import { App } from "./app/App";

wasm_init();

const root = document.getElementById("root");
root.innerHTML = "";
if (!root) {
  throw new Error("expect #root");
}
render(() => <App />, root);
