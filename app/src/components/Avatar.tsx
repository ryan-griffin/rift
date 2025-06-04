import { Component } from "solid-js";
import { Avatar as ArkAvatar } from "@ark-ui/solid/avatar";

const Avatar: Component<
	{ fallback?: string; src?: string; className?: string }
> = (props) => {
	return (
		<ArkAvatar.Root
			class={`aspect-square rounded-full bg-background-100 dark:bg-background-800 flex items-center justify-center ${props.className}`}
		>
			<ArkAvatar.Fallback class="font-bold">
				{props.fallback || "?"}
			</ArkAvatar.Fallback>
			<ArkAvatar.Image src={props.src} alt="avatar" />
		</ArkAvatar.Root>
	);
};

export default Avatar;
