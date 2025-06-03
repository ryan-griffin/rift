import { Component } from "solid-js";
import { Field } from "@ark-ui/solid/field";

interface Props {
	placeholder?: string;
	value?: string;
	onInput?: (value: string) => void;
	onKeyDown?: (e: KeyboardEvent) => void;
	className?: string;
}

const Input: Component<Props> = (props) => {
	const style =
		"rounded-lg p-2 placeholder-background-400 bg-background-100 dark:bg-background-800 dark:placeholder-background-500 focus:outline-2 -outline-offset-1 outline-accent-500 transition-colors duration-200 w-full";

	return (
		<Field.Root class={props.className}>
			<Field.Input
				placeholder={props.placeholder}
				value={props.value}
				onInput={(e) => props.onInput?.(e.currentTarget.value)}
				onKeyDown={props.onKeyDown}
				class={style}
			/>
		</Field.Root>
	);
};

export default Input;
