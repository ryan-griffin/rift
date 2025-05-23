import { Component, JSX } from "solid-js";
import { Splitter as ArkSplitter } from "@ark-ui/solid/splitter";

interface Props {
	a: JSX.Element;
	b: JSX.Element;
}

const Splitter: Component<Props> = (props) => (
	<ArkSplitter.Root
		defaultSize={[30, 70]}
		panels={[{ id: "a", minSize: 25 }, { id: "b", minSize: 60 }]}
	>
		<ArkSplitter.Panel id="a">{props.a}</ArkSplitter.Panel>
		<ArkSplitter.ResizeTrigger
			class="w-2"
			id="a:b"
			aria-label="Resize"
		/>
		<ArkSplitter.Panel id="b">{props.b}</ArkSplitter.Panel>
	</ArkSplitter.Root>
);

export default Splitter;
