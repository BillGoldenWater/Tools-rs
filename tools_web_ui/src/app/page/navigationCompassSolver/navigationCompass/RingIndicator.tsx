import { Motion } from "@motionone/solid";
import { spring } from "motion";

export interface RingIndicatorProps {
  offset: number;
  current: number;
  active: boolean;
}

export function RingIndicator(props: RingIndicatorProps) {
  const xOffset = () => -0.25 * (8 * props.offset + 8);
  const rotate = () => props.current * 60;

  return (
    <>
      <Motion.div
        animate={{ rotate: rotate() }}
        transition={{
          easing: spring({
            damping: 10,
            mass: 1,
          }),
        }}
        class={"group absolute left-24 top-24 h-0 w-0 transform-gpu"}
        classList={{ ringIndicatorActive: props.active }}
      >
        <div
          style={{
            "--tw-translate-x": `${xOffset()}rem`,
          }}
          class={
            "absolute h-2 w-8 -translate-y-1 transform-gpu rounded bg-cyan-700 group-[.ringIndicatorActive]:bg-cyan-500 dark:bg-cyan-900 dark:group-[.ringIndicatorActive]:bg-cyan-300"
          }
        />
      </Motion.div>
    </>
  );
}
