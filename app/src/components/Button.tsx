import { Component, JSX, Show } from "solid-js";

interface Props {
	type?: "button" | "submit" | "reset";
	variant: "regular" | "flat" | "suggested" | "destructive";
	rounded?: boolean;
	icon?: JSX.Element;
	text?: string;
	title?: string;
	disabled?: boolean;
	onClick?: JSX.EventHandler<HTMLButtonElement, MouseEvent>;
	className?: string;
}

const Button: Component<Props> = (props) => {
	const { icon } = props;

	const variantStyles = {
		regular:
			"bg-background-100 hover:bg-background-200 active:bg-background-300 dark:bg-background-800 dark:hover:bg-background-700 dark:active:bg-background-600",
		flat:
			"hover:bg-background-100 active:bg-background-200 dark:hover:bg-background-800 dark:active:bg-background-700",
		suggested:
			"text-white bg-accent-500 hover:bg-accent-400 active:bg-accent-600",
		destructive:
			"text-red-400 bg-red-400/20 hover:bg-red-400/30 active:bg-red-400/40",
	};

	const contentStyles = () => {
		if (icon && props.text) {
			return `flex gap-2 items-center justify-center ${
				props.rounded ? "pl-4 pr-5 py-3" : "pl-2 pr-3 py-2"
			}`;
		}

		if (icon && !props.text) {
			return "flex items-center justify-center p-2";
		}

		return props.rounded ? "px-5 py-3" : "px-3 py-2";
	};

	const buttonClass = () => {
		const classes = [
			"cursor-pointer transition-colors duration-200",
			props.text ? "font-bold" : "",
			props.rounded ? "rounded-full" : "rounded-lg",
			contentStyles(),
			variantStyles[props.variant],
			props.className || "",
		];

		return classes.filter(Boolean).join(" ");
	};

	return (
		<button
			class={buttonClass()}
			type={props.type}
			title={props.title}
			disabled={props.disabled}
			onClick={props.onClick}
		>
			<Show when={icon}>{icon}</Show>
			<Show when={props.text}>{props.text}</Show>
		</button>
	);
};

export default Button;
