// @refresh reload
import { createHandler, StartServer } from "@solidjs/start/server";

export default createHandler(() => (
	<StartServer
		document={({ assets, children, scripts }) => (
			<html lang="en">
				<head>
					<meta charset="utf-8" />
					<meta
						name="viewport"
						content="width=device-width, initial-scale=1"
					/>
					{assets}
				</head>
				<body class="bg-background-100 text-background-950 dark:text-background-50 dark:bg-background-950">
					<div id="app">{children}</div>
					{scripts}
				</body>
			</html>
		)}
	/>
));
