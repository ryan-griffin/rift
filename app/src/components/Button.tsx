import { isServer } from "solid-js/web";

export const Button = () => {
	return (
		<button type="button" class="p-2 bg-blue-500 text-white">
			{isServer ? "Server" : "Client"}
		</button>
	);
};
