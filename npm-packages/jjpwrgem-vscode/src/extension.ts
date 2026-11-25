import * as vscode from "vscode";
import { getSetting } from "./settings";
import {
  describeStartError,
  INSTALL_ACTION,
  SETTINGS_ACTION,
} from "./startError";
import {
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext): {
  whenReady: Promise<void>;
} {
  const command = getSetting("jjpwrgem.path").trim() || "jjp";
  const custom = command !== "jjp";
  const serverOptions: ServerOptions = {
    command,
    args: ["lsp"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", language: "json" },
      { scheme: "file", language: "jsonc" },
    ],
    synchronize: { configurationSection: "jjpwrgem" },
    initializationOptions: {
      format: vscode.workspace.getConfiguration("jjpwrgem").get("format"),
    },
  };

  client = new LanguageClient(
    "jjpwrgem",
    "jjpwrgem LSP",
    serverOptions,
    clientOptions,
  );

  context.subscriptions.push(client);
  const whenReady = client.start().catch((error: unknown) => {
    const { message, actions } = describeStartError(command, custom, error);
    void vscode.window.showErrorMessage(message, ...actions).then((action) => {
      if (action === INSTALL_ACTION) {
        void vscode.env.openExternal(
          vscode.Uri.parse("https://github.com/20jasper/jjpwrgem#installation"),
        );
      } else if (action === SETTINGS_ACTION) {
        void vscode.commands.executeCommand(
          "workbench.action.openSettings",
          "jjpwrgem.path",
        );
      }
    });
    throw error;
  });
  whenReady.catch(() => undefined);
  return { whenReady };
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
