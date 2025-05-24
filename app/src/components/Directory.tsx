import { Component, createResource, For, Show, Suspense } from "solid-js";
import { createTreeCollection, TreeView } from "@ark-ui/solid/tree-view";
import ChevronRight from "../assets/chevron-right.svg";
import MessageSquareText from "../assets/message-square-text.svg";

interface Node {
	id: number;
	name: string;
	type: "folder" | "channel";
	parent_id: number | null;
}

interface DirectoryNode {
	id: number;
	name: string;
	type: "folder" | "channel";
	children?: DirectoryNode[];
}

const buildDirectory = (nodes: Node[]): DirectoryNode | undefined => {
	const nodeMap = new Map<number, DirectoryNode>();

	for (const node of nodes) {
		nodeMap.set(node.id, {
			id: node.id,
			name: node.name,
			type: node.type,
		});
	}

	const root = nodes.find((node) => node.parent_id === null);
	if (!root) return;

	for (const node of nodes) {
		if (node.parent_id !== null) {
			const parent = nodeMap.get(node.parent_id);
			const child = nodeMap.get(node.id);

			if (parent && child) {
				if (!parent.children) parent.children = [];
				parent.children.push(child);
			}
		}
	}

	return nodeMap.get(root.id);
};

const DirectoryItem: Component<TreeView.NodeProviderProps<DirectoryNode>> = (
	props,
) => {
	const { node, indexPath } = props;
	const nodeClass =
		"flex p-2 gap-2 rounded-lg hover:bg-background-200 dark:hover:bg-background-800 transition-colors duration-100 cursor-pointer select-none";

	return (
		<TreeView.NodeProvider node={node} indexPath={indexPath}>
			<Show
				when={node.type === "folder"}
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
					<TreeView.BranchContent class="flex gap-3 overflow-hidden data-[state=closed]:animate-[slideUp_200ms] data-[state=open]:animate-[slideDown_200ms]">
						<TreeView.BranchIndentGuide class="border-l-2 border-background-200 dark:border-background-900 ml-4" />
						<div class="flex flex-col gap-1 grow">
							<For each={node.children}>
								{(child, index) => (
									<DirectoryItem
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
	const [nodes] = createResource<Node[]>(async () => {
		const res = await fetch("http://localhost:3000/api/directory/1");
		return res.json();
	});

	return (
		<Suspense fallback={<p>Loading...</p>}>
			<Show when={nodes()}>
				{(nodesData) => {
					const directory = buildDirectory(nodesData());
					if (!directory) return;

					return (
						<TreeView.Root
							collection={createTreeCollection<DirectoryNode>({
								nodeToValue: (node) => node.id.toString(),
								nodeToString: (node) => node.name,
								rootNode: directory,
							})}
						>
							<TreeView.Tree class="flex flex-col p-2 gap-1">
								<For each={directory.children}>
									{(node, index) => (
										<DirectoryItem
											node={node}
											indexPath={[index()]}
										/>
									)}
								</For>
							</TreeView.Tree>
						</TreeView.Root>
					);
				}}
			</Show>
		</Suspense>
	);
};

export default Directory;
