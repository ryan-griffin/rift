import { Component, For, Show } from "solid-js";
import { createTreeCollection, TreeView } from "@ark-ui/solid/tree-view";
import ChevronRight from "../assets/chevron-right.svg";
import MessageSquareText from "../assets/message-square-text.svg";

interface Node {
	id: string;
	name: string;
	children?: Node[];
}

const collection = createTreeCollection<Node>({
	nodeToValue: (node) => node.id,
	nodeToString: (node) => node.name,
	rootNode: {
		id: "ROOT",
		name: "",
		children: [
			{
				id: "general",
				"name": "General",
			},
			{
				id: "gaming",
				name: "Gaming",
				children: [
					{ id: "roblox", name: "Roblox" },
					{ id: "fortnite", name: "Fortnite" },
					{
						id: "minecraft",
						name: "Minecraft",
						children: [
							{ id: "mods", name: "Mods" },
							{
								id: "modpacks",
								name: "Modpacks",
							},
						],
					},
				],
			},
			{
				id: "programming",
				name: "Programming",
				children: [
					{ id: "typescript", name: "TypeScript" },
					{ id: "rust", name: "Rust" },
				],
			},
			{ id: "announcements", name: "Announcements" },
			{ id: "memes", name: "Memes" },
			{ id: "help", name: "Help" },
		],
	},
});

const TreeNode: Component<TreeView.NodeProviderProps<Node>> = (props) => {
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
							class={`${nodeClass} group`}
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
									<TreeNode
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

const Tree = () => {
	return (
		<TreeView.Root
			collection={collection}
			class="flex flex-col h-screen p-4 gap-4 bg-background-50 dark:bg-background-900"
		>
			<TreeView.Label>Rift</TreeView.Label>
			<TreeView.Tree class="flex flex-col gap-1">
				<For each={collection.rootNode.children}>
					{(node, index) => (
						<TreeNode node={node} indexPath={[index()]} />
					)}
				</For>
			</TreeView.Tree>
		</TreeView.Root>
	);
};

export default Tree;
