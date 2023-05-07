import { NavigationCompass } from "./navigationCompassSolver/NavigationCompass";
import { createSignal, For, Setter, Show, Signal } from "solid-js";
import { Panel } from "../../component/Panel";
import { navigation_compass_solve } from "tools_wasm";

type Amount = 1 | 2 | 3 | 4;
type Direction = -1 | 1;

export function NavigationCompassSolver() {
  const [inner, setInner] = createSignal(4);
  const [middle, setMiddle] = createSignal(0);
  const [outer, setOuter] = createSignal(0);
  const [innerN, setInnerN] = createSignal<Amount>(1);
  const [middleN, setMiddleN] = createSignal<Amount>(3);
  const [outerN, setOuterN] = createSignal<Amount>(3);
  const [innerD, setInnerD] = createSignal<Direction>(-1);
  const [middleD, setMiddleD] = createSignal<Direction>(1);
  const [outerD, setOuterD] = createSignal<Direction>(1);

  const linkages = [
    createSignal<number>(0b011),
    createSignal<number>(0b101),
    createSignal<number>(0b110),
  ];

  const [result, setResult] = createSignal<null | string | number[]>(null);

  function solve() {
    let navigationCompassResult = navigation_compass_solve(
      inner(),
      innerN(),
      innerD(),
      middle(),
      middleN(),
      middleD(),
      outer(),
      outerN(),
      outerD(),
      new Uint8Array(linkages.map((it) => it[0]()))
    );

    if (navigationCompassResult != null) {
      setResult(Array.from(navigationCompassResult));
    } else {
      setResult("无法在 100 步内得出结果");
    }
  }

  function reset() {
    setInner(0);
    setInnerN(1);
    setInnerD(1);

    setMiddle(0);
    setMiddleN(1);
    setMiddleD(1);

    setOuter(0);
    setOuterN(1);
    setOuterD(1);

    linkages.forEach(([val, set], idx) => set(0b100 >> idx));
    setResult(null);
  }

  return (
    <div class={"grid justify-center gap-4 p-4"}>
      <Panel class={"flex items-center justify-center"}>
        <NavigationCompass
          innerCurrent={inner()}
          middleCurrent={middle()}
          outerCurrent={outer()}
        />
      </Panel>
      <Panel>
        <div class={"flex flex-col"}>
          <datalist id="navigationCompassRingValues">
            <option value="0" />
            <option value="1" />
            <option value="2" />
            <option value="3" />
            <option value="4" />
            <option value="5" />
          </datalist>
          <div class={"flex gap-2"}>
            <span>内:</span>
            <CurrentValueSelector value={inner()} setter={setInner} />
            方向: <DirectionSelector value={innerD()} setter={setInnerD} />
            格数: <AmountSelector value={innerN()} setter={setInnerN} />
          </div>
          <div class={"flex gap-2"}>
            <span>中:</span>
            <CurrentValueSelector value={middle()} setter={setMiddle} />
            方向: <DirectionSelector value={middleD()} setter={setMiddleD} />
            格数: <AmountSelector value={middleN()} setter={setMiddleN} />
          </div>
          <div class={"flex gap-2"}>
            <span>外:</span>
            <CurrentValueSelector value={outer()} setter={setOuter} />
            方向: <DirectionSelector value={middleD()} setter={setOuterD} />
            格数: <AmountSelector value={outerN()} setter={setOuterN} />
          </div>
        </div>
      </Panel>
      <LinkageSelector values={linkages} />
      <Panel class={"flex gap-4"}>
        <button class={"w-full"} onclick={() => solve()}>
          推演
        </button>
        <button class={"w-full"} onclick={() => reset()}>
          重置
        </button>
      </Panel>
      <ResultDisplayer result={result()} />
    </div>
  );
}

function LinkageSelector(props: { values: Signal<number>[] }) {
  return (
    <Panel class={"flex flex-col gap-4"}>
      <div class={"flex justify-center"}>联动信息 (游戏内屏幕下方)</div>
      <For each={props.values}>
        {([val, set]) => <LinkageItem value={val()} setter={set} />}
      </For>
    </Panel>
  );
}

function ResultDisplayer(props: { result: null | string | number[] }) {
  return (
    <Show when={props.result != null} fallback={<></>}>
      <Panel class={"flex flex-col gap-4"}>
        <div class={"flex justify-center"}>结果</div>
        <Show
          when={typeof props.result !== "string"}
          fallback={<span>{props.result}</span>}
        >
          <For each={props.result as number[]}>
            {(val) => <LinkageItem value={val} />}
          </For>
        </Show>
      </Panel>
    </Show>
  );
}

function LinkageItem(props: { value: number; setter?: Setter<number> }) {
  function flip(idx: 0 | 1 | 2) {
    const mask = 0b100 >> idx;
    const invMask = ~mask;

    props.setter((p) => (~p & mask) | (p & invMask));
  }

  return (
    <div
      class={"flex justify-around rounded-lg bg-gray-200 p-2 dark:bg-gray-900"}
    >
      <LinkageItemDot
        value={(props.value & 0b100) > 0}
        flip={props.setter && flip.bind(null, 0)}
      />
      <LinkageItemDot
        value={(props.value & 0b010) > 0}
        flip={props.setter && flip.bind(null, 1)}
      />
      <LinkageItemDot
        value={(props.value & 0b001) > 0}
        flip={props.setter && flip.bind(null, 2)}
      />
    </div>
  );
}

function LinkageItemDot(props: { value: boolean; flip?: () => void }) {
  return (
    <Show
      when={props.value}
      fallback={
        <div
          class={
            "h-4 w-4 rounded-xl border-2 border-solid border-gray-300 bg-gray-200 transition-colors dark:border-gray-500 dark:bg-gray-800"
          }
          onclick={props.flip}
        />
      }
    >
      <div
        class={
          "h-4 w-4 rounded-xl bg-orange-500 transition-colors hover:bg-orange-400"
        }
        onclick={props.flip}
      />
    </Show>
  );
}

function CurrentValueSelector(props: {
  value: number;
  setter: Setter<number>;
}) {
  return (
    <input
      type={"range"}
      list={"navigationCompassRingValues"}
      min={0}
      max={5}
      step={1}
      value={props.value}
      onInput={(e) => props.setter(Number.parseInt(e.target.value))}
    />
  );
}

function DirectionSelector(props: { value: 1 | -1; setter: Setter<1 | -1> }) {
  return (
    <select
      value={props.value.toString()}
      onchange={(e) => props.setter(e.target.value === "1" ? 1 : -1)}
    >
      <option value={"1"}>顺时针</option>
      <option value={"-1"}>逆时针</option>
    </select>
  );
}

function AmountSelector(props: { value: Amount; setter: Setter<Amount> }) {
  return (
    <select
      value={props.value.toString()}
      onchange={(e) => props.setter(Number.parseInt(e.target.value) as Amount)}
    >
      <option value={"1"}>1</option>
      <option value={"2"}>2</option>
      <option value={"3"}>3</option>
      <option value={"4"}>4</option>
    </select>
  );
}
