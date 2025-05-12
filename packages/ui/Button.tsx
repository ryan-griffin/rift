import { isServer } from "solid-js/web";

export const Button = () => {
	return <button type="button">{isServer ? "Server" : "Client"}</button>;
};
