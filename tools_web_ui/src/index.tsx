/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";

import { App } from "./app/App";
import { wasm_init } from "tools_wasm";

wasm_init();

const root = document.getElementById("root");
root.innerHTML = "";
render(() => <App />, root!);
