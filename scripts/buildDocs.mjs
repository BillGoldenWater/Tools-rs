import {execSync} from "child_process";
import fs from "fs";

/**
 * @param command {string}
 * @param cwd {string}
 */
function execCommand(command, cwd = undefined) {
  console.log(`running ${command}`);
  return execSync(command, {cwd, stdio: "inherit"});
}

/**
 * @param {string} msg
 */
function highlightLog(msg) {
  console.log(`\u001b[94m${msg}\u001b[0m`);
}


async function main() {
  highlightLog("building wasm");
  execCommand("rm -rf pkg", "tools_wasm");
  execCommand("cargo install wasm-pack", "tools_wasm");
  execCommand("wasm-pack build", "tools_wasm");

  highlightLog("building js");
  execCommand("rm -rf node_modules/tools_wasm", "tools_web_ui");
  execCommand("yarn --check-files", "tools_web_ui");
  execCommand("npx vite build --force", "tools_web_ui");

  // move to docs
  highlightLog("moving to docs");
  if (fs.existsSync("docs")) execCommand("mv docs docs.old");
  else execCommand("mkdir docs.old");
  execCommand("mv tools_web_ui/dist docs");

  // restore cname
  highlightLog("restore CNAME");
  execCommand("touch docs.old/CNAME");
  execCommand("mv docs.old/CNAME docs/CNAME");

  execCommand("rm -rf docs.old");

  highlightLog("add new files to git");
  execCommand("git add docs")
}

await main();