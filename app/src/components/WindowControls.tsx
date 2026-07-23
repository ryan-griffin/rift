import { getCurrentWindow } from "@tauri-apps/api/window";
import Logo from "../assets/logo.svg";
import Minus from "../assets/minus.svg";
import Square from "../assets/square.svg";
import X from "../assets/x.svg";
import Button from "./Button.tsx";

const WindowControls = () => {
	const currentWindow = getCurrentWindow();

	return (
		<div class="w-full flex p-1 select-none">
			<div
				role="none"
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
				<div class="flex items-center p-2 gap-2">
					<Logo />
					<p class="font-bold">Rift</p>
				</div>
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
