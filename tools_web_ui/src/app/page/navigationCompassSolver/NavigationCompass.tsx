import { RingIndicator } from "./navigationCompass/RingIndicator";

export interface NavigationCompassProps {
  innerCurrent: number;
  middleCurrent: number;
  outerCurrent: number;
}

export function NavigationCompass(props: NavigationCompassProps) {
  return (
    <div class={"p-8"}>
      <div
        class={
          "relative aspect-square h-48 rounded-[50%] bg-neutral-300 dark:bg-neutral-600"
        }
      >
        <RingIndicator offset={0} current={props.innerCurrent} active={true} />
        <RingIndicator offset={1} current={props.middleCurrent} active={true} />
        <RingIndicator offset={2} current={props.outerCurrent} active={true} />

        <RingIndicator offset={3} current={0} active={true} />
        <RingIndicator offset={3} current={1} active={false} />
        <RingIndicator offset={3} current={2} active={false} />
        <RingIndicator offset={3} current={3} active={false} />
        <RingIndicator offset={3} current={4} active={false} />
        <RingIndicator offset={3} current={5} active={false} />
      </div>
    </div>
  );
}
