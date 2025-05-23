import { getCurrentWindow } from "@tauri-apps/api/window";
import Button from "./Button.tsx";
import Minus from "../assets/minus.svg";
import Square from "../assets/square.svg";
import X from "../assets/x.svg";

const WindowControls = () => {
	const currentWindow = getCurrentWindow();

	return (
		<div class="w-full flex p-1 select-none">
			<div
				data-tauri-drag-region
				class="w-full"
				onMouseDown={(e) => {
					if (e.buttons === 1) {
						e.detail === 2
							? currentWindow.toggleMaximize()
							: currentWindow.startDragging();
					}
				}}
			>
				<p class="p-2 font-bold">Rift</p>
			</div>
			<Button
				variant="flat"
				type="button"
				icon={<Minus />}
				onClick={() => currentWindow.minimize()}
			/>
			<Button
				variant="flat"
				type="button"
				icon={<Square />}
				onClick={() => currentWindow.toggleMaximize()}
			/>
			<Button
				variant="flat"
				type="button"
				icon={<X />}
				onClick={() => currentWindow.close()}
			/>
		</div>
	);
};

export default WindowControls;
