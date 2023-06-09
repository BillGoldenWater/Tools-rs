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
        <div class={"flex flex-col gap-2 break-keep"}>
          <datalist id="navigationCompassRingValues">
            <option value="0" />
            <option value="1" />
            <option value="2" />
            <option value="3" />
            <option value="4" />
            <option value="5" />
          </datalist>
          <RingInfoInput
            name={"内圈"}
            current={inner()}
            setCurrent={setInner}
            direction={innerD()}
            setDirection={setInnerD}
            amount={innerN()}
            setAmount={setInnerN}
          />
          <RingInfoInput
            name={"中圈"}
            current={middle()}
            setCurrent={setMiddle}
            direction={middleD()}
            setDirection={setMiddleD}
            amount={middleN()}
            setAmount={setMiddleN}
          />
          <RingInfoInput
            name={"外圈"}
            current={outer()}
            setCurrent={setOuter}
            direction={outerD()}
            setDirection={setOuterD}
            amount={outerN()}
            setAmount={setOuterN}
          />
        </div>
      </Panel>
      <LinkageSelector values={linkages} result={result()} />
      <Panel class={"flex gap-4"}>
        <button class={"w-full"} onclick={() => solve()}>
          推演
        </button>
        <button class={"w-full"} onclick={() => reset()}>
          重置
        </button>
      </Panel>
    </div>
  );
}

function RingInfoInput(props: {
  name: string;
  current: number;
  setCurrent: Setter<number>;
  direction: Direction;
  setDirection: Setter<Direction>;
  amount: Amount;
  setAmount: Setter<Amount>;
}) {
  return (
    <div class={"flex flex-wrap gap-2"}>
      <div class={"flex gap-2"}>
        {props.name}:
        <CurrentValueSelector value={props.current} setter={props.setCurrent} />
      </div>
      <div class={"flex gap-2"}>
        方向:
        <DirectionSelector
          value={props.direction}
          setter={props.setDirection}
        />
      </div>
      <div class={"flex gap-2"}>
        格数:
        <AmountSelector value={props.amount} setter={props.setAmount} />
      </div>
    </div>
  );
}

function LinkageSelector(props: {
  values: Signal<number>[];
  result: null | string | number[];
}) {
  function resultCounter(value: number, result: number[]) {
    let count = 0;
    for (const val of result) {
      if (value == val) count += 1;
    }
    return <>{count}</>;
  }

  return (
    <Panel class={"flex flex-col gap-4"}>
      <div class={"flex justify-center"}>
        联动信息 (游戏内屏幕下方){" "}
        <Show when={props.result != null && typeof props.result !== "string"}>
          x 旋转次数
        </Show>
      </div>
      <Show when={typeof props.result === "string"}>
        <div class={"flex justify-center text-red-800 dark:text-red-300"}>
          {props.result}
        </div>
      </Show>
      <For each={props.values}>
        {([val, set]) => (
          <div class={"flex gap-2"}>
            <LinkageItem value={val()} setter={set} />
            <Show
              when={props.result != null && typeof props.result !== "string"}
            >
              <div
                class={
                  "flex items-center rounded-lg bg-gray-200 pl-4 pr-4 font-mono dark:bg-gray-900"
                }
              >
                {resultCounter(val(), props.result as number[])}
              </div>
            </Show>
          </div>
        )}
      </For>
    </Panel>
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
      class={
        "flex flex-grow justify-around rounded-lg bg-gray-200 p-2 dark:bg-gray-900"
      }
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
