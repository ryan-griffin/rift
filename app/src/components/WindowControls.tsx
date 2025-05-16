import { getCurrentWindow } from "@tauri-apps/api/window";
import Minus from "../assets/minus.svg?component-solid";
import Square from "../assets/square.svg?component-solid";
import X from "../assets/x.svg?component-solid";

const WindowControls = () => {
	const currentWindow = getCurrentWindow();

	return (
		<div class="sticky top-0 flex p-1 justify-end bg-gray-100 select-none">
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
			/>
			<button
				type="button"
				class="select-none p-1"
				onClick={() => currentWindow.minimize()}
			>
				<Minus />
			</button>
			<button
				type="button"
				class="select-none p-1"
				onClick={() => currentWindow.toggleMaximize()}
			>
				<Square />
			</button>
			<button
				type="button"
				class="select-none p-1"
				onClick={() => currentWindow.close()}
			>
				<X />
			</button>
		</div>
	);
};

export default WindowControls;
