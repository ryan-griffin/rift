import { Component, Index } from "solid-js";
import { SegmentGroup as ArkSegmentGroup } from "@ark-ui/solid/segment-group";

interface Props {
	value: string;
	setValue: (value: string) => void;
	items: string[];
	onChange?: (value: string) => void;
	className?: string;
}

const SegmentGroup: Component<Props> = (props) => {
	return (
		<ArkSegmentGroup.Root
			value={props.value}
			onValueChange={(e) => {
				props.setValue(e.value!);
				props.onChange?.(e.value!);
			}}
			class={`flex p-1 gap-1 rounded-xl bg-background-100 dark:bg-background-800 ${props.className}`}
		>
			<ArkSegmentGroup.Indicator class="left-[var(--left)] w-[var(--width)] h-[var(--height)] rounded-lg bg-background-50 dark:bg-background-700" />
			<Index each={props.items}>
				{(item) => (
					<ArkSegmentGroup.Item
						value={item()}
						class="z-10 flex-1 p-2 text-center rounded-lg cursor-pointer"
					>
						<ArkSegmentGroup.ItemText>{item()}</ArkSegmentGroup.ItemText>
						<ArkSegmentGroup.ItemControl />
						<ArkSegmentGroup.ItemHiddenInput />
					</ArkSegmentGroup.Item>
				)}
			</Index>
		</ArkSegmentGroup.Root>
	);
};

export default SegmentGroup;
