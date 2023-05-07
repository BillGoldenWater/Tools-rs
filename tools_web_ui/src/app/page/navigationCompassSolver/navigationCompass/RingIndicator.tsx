import { Motion } from "@motionone/solid";
import { spring } from "motion";

export interface RingIndicatorProps {
  offset: number;
  current: number;
  active: boolean;
  isInner: boolean;
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
            damping: 15,
            mass: 1,
            stiffness: 150,
          }),
        }}
        class={"group absolute left-24 top-24 h-0 w-0 transform-gpu"}
        classList={{
          ringIndicatorNonActive: !props.active,
          ringIndicatorIsInner: props.isInner,
        }}
      >
        <div
          style={{
            "--tw-translate-x": `${xOffset()}rem`,
          }}
          class={
            "absolute h-2 w-8 -translate-y-1 transform-gpu rounded bg-cyan-400 group-[.ringIndicatorIsInner]:hue-rotate-[200deg] group-[.ringIndicatorNonActive]:saturate-50 dark:bg-cyan-300 dark:group-[.ringIndicatorNonActive]:brightness-75"
          }
        />
      </Motion.div>
    </>
  );
}
