import "./AeroplaneChessSolver.css";
import { createSignal, For, JSX, onCleanup, onMount, Show } from "solid-js";
import { AdvancePreview, Board } from "tools_wasm";
import { Panel } from "../../component/Panel";

export function AeroplaneChessSolver() {
  const [board, setBoard] = createSignal({ it: Board.new() });
  const [step, setStep] = createSignal(1);
  const [keyTarget, setKeyTarget] = createSignal<"step" | "advance">("step");

  function onKey(e: KeyboardEvent) {
    if (e.key >= "1" && e.key <= "6") {
      let key = Number.parseInt(e.key);
      if (keyTarget() == "step") {
        setStep(key);
        setKeyTarget("advance");
      } else if (key <= 4) {
        const len = board().it.flying_len();

        if (board().it.hangar_empty()) {
          if (key <= len) {
            board().it.advance(key - 1, step());
          }
        } else if (key <= len + 1) {
          if (key === len + 1) {
            board().it.takeoff(step());
          } else {
            board().it.advance(key - 1, step());
          }
        }

        setKeyTarget("step");
        update();
      }
    } else if (e.key === "Escape") {
      setKeyTarget("step");
    }
  }

  onMount(() => {
    window.addEventListener("keyup", onKey);
  })

  onCleanup(() => {
    board().it.free();
    window.removeEventListener("keyup", onKey);
  })

  function update() {
    setBoard((prev) => ({ ...prev }));
  }

  return (
    <div class="grid justify-center gap-4 p-4 grid-cols-[auto_auto] grid-rows-[min-content_auto]">
      <Panel class="min-w-[14ch]">
        <div>机库: {board().it.hangar()}</div>
        <div>完成: {board().it.done()}</div>
        <div>
          按键: {
            keyTarget() === "step"
              ? "点数"
              : <span class="text-orange-300">走</span>
          }
        </div>
      </Panel>
      <Panel class="row-span-2 grid gap-2 grid-cols-2 grid-rows-3 place-self-start">
        <For each={[1, 2, 3, 4, 5, 6]}>
          {(it) => (
            <Show
              when={step() === it}
              fallback={
                <div
                  class="ACS_StepSelector border-gray-700"
                  onclick={() => {
                    setStep(it)
                  }}
                >
                  {it}
                </div>
              }
            >
              <div class="ACS_StepSelector border-orange-300">
                {it}
              </div>
            </Show>
          )}
        </For>
      </Panel>
      <Panel class="flex flex-col gap-2 min-h-30 w-218">
        {((it) => {
          using previews = it;
          const hangar = it.takeoff != null;
          const flying = board().it.flying();
          const flying_len = flying.length;

          return <>
            <For each={previews.flying}>
              {(it, idx) => {
                using chess = flying[idx()];

                return (
                  <Preview
                    id={idx() + 1}
                    pos={chess.pos()}
                    preview={it}
                    click={() => {
                      board().it.advance(idx(), step());
                      update();
                    }}
                    ret={() => {
                      board().it.ret(idx())
                      update();
                    }}
                  />
                );
              }}
            </For>
            <Show when={hangar}>
              <Preview
                id={flying_len + 1}
                pos={0}
                preview={it.takeoff}
                click={() => {
                  board().it.takeoff(step());
                  update();
                }}
                ret={() => { }}
              />
            </Show>
          </>;
        })(board().it.preview(step()))}
        {/* <button */}
        {/*   onclick={() => { */}
        {/*     if (!board().it.hangar_empty()) { */}
        {/*       board().it.takeoff(1); */}
        {/*       update() */}
        {/*     } */}
        {/*   }} */}
        {/* > */}
        {/*   Takeoff */}
        {/* </button> */}
        {/* {step()} */}
      </Panel>
    </div >
  );
}

/// take ownership of preview
function Preview(p: {
  id: number,
  pos: number,
  preview: AdvancePreview,
  click: () => void,
  ret: () => void,
}) {
  using preview = p.preview;

  function displayBool(b: boolean): JSX.Element {
    return b ? <span class="text-green-500">是</span> : "否"
  }

  function probabilityStr(p: number): JSX.Element {
    const sign = p >= -0 ? "+" : "-";
    const percent = (Math.abs(p) * (1 / 6) * 100).toFixed(2);
    const text = `${sign}${percent}%`.padStart(7, " ");
    return (
      p > 0.01
        ? <span class="text-green-500">{text}</span>
        : p < -0.01
          ? <span class="text-red-500">{text}</span>
          : text);
  }

  return (
    <div class="ACS_Preview">
      <div>
        编号: {p.id.toString()}
      </div>
      <div>
        位置: {p.pos.toString().padStart(2, " ")}
      </div>
      <div>
        新位置: {preview.pos.toString().padStart(2, " ")}
      </div>
      <div>
        跳: {displayBool(preview.will_jump)}
      </div>
      <div>
        终点: {displayBool(preview.will_enter_end)}
      </div>
      <div>
        完成: {displayBool(preview.will_done)}
      </div>
      <div>
        跳: {probabilityStr(preview.jump_change)}
      </div>
      <div>
        完成: {probabilityStr(preview.done_change)}
      </div>
      <button onclick={p.click}>走</button>
      <Show when={p.pos !== 0 || true}>
        <button onclick={p.ret}>回</button>
      </Show>
    </div>
  );
}
