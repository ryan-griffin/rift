import { Component, createResource, For, Show, Suspense } from "solid-js";
import { createTreeCollection, TreeView } from "@ark-ui/solid/tree-view";
import ChevronRight from "../assets/chevron-right.svg";
import MessageSquareText from "../assets/message-square-text.svg";

interface Node {
	id: number;
	name: string;
	children?: Node[];
}

const DirectoryNode: Component<TreeView.NodeProviderProps<Node>> = (props) => {
	const { node, indexPath } = props;
	const nodeClass =
		"flex p-2 gap-2 rounded-lg hover:bg-background-100 dark:hover:bg-background-800 transition-colors duration-100 cursor-pointer select-none";

	return (
		<TreeView.NodeProvider node={node} indexPath={indexPath}>
			<Show
				when={node.children}
				fallback={
					<TreeView.Item>
						{
							/* <TreeView.ItemIndicator>
							<Square />
						</TreeView.ItemIndicator> */
						}
						<TreeView.ItemText
							class={`${nodeClass} data-[selected]:bg-accent-100 data-[selected]:hover:bg-accent-100 data-[selected]:dark:bg-accent-800 data-[selected]:dark:hover:bg-accent-800`}
						>
							<MessageSquareText />
							{node.name}
						</TreeView.ItemText>
					</TreeView.Item>
				}
			>
				<TreeView.Branch class="flex flex-col gap-1">
					<TreeView.BranchControl>
						<TreeView.BranchText
							class={`${nodeClass} group text-background-400 dark:text-background-500 font-bold`}
						>
							<ChevronRight class="group-data-[state=open]:rotate-90 transition-transform duration-200" />
							{node.name}
						</TreeView.BranchText>
						{
							/* <TreeView.BranchIndicator>
							<Square />
						</TreeView.BranchIndicator> */
						}
					</TreeView.BranchControl>
					<TreeView.BranchContent class="flex gap-3">
						<TreeView.BranchIndentGuide class="border-l-2 border-background-100 dark:border-background-800 ml-4" />
						<div class="flex flex-col gap-1 grow">
							<For each={node.children}>
								{(child, index) => (
									<DirectoryNode
										node={child}
										indexPath={[...indexPath, index()]}
									/>
								)}
							</For>
						</div>
					</TreeView.BranchContent>
				</TreeView.Branch>
			</Show>
		</TreeView.NodeProvider>
	);
};

const Directory = () => {
	const [directory] = createResource<Node>(async () => {
		const res = await fetch("http://localhost:3000/api/directory");
		return res.json();
	});

	return (
		<Suspense fallback={<div>Loading...</div>}>
			<Show when={directory()}>
				{(directoryData) => (
					<TreeView.Root
						collection={createTreeCollection<Node>({
							nodeToValue: (node) => node.id.toString(),
							nodeToString: (node) => node.name,
							rootNode: directoryData(),
						})}
					>
						<TreeView.Tree class="flex flex-col p-4 pt-0 gap-1 overflow-auto">
							<For each={directoryData().children}>
								{(node, index) => (
									<DirectoryNode
										node={node}
										indexPath={[index()]}
									/>
								)}
							</For>
						</TreeView.Tree>
					</TreeView.Root>
				)}
			</Show>
		</Suspense>
	);
};

export default Directory;
