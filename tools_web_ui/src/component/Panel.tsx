import { ParentProps } from "solid-js";

export function Panel(props: ParentProps & { class?: string }) {
  const additionalClass = () => (props.class != null ? ` ${props.class}` : "");

  return (
    <div
      class={
        "rounded-xl bg-gray-100 p-4 shadow-xl transition-shadow hover:shadow-2xl focus:shadow-2xl dark:bg-gray-800" +
        additionalClass()
      }
    >
      {props.children}
    </div>
  );
}
