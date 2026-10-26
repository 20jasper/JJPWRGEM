import * as vscode from "vscode";
import { getSetting } from "./settings";
import {
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext): {
  whenReady: Promise<void>;
} {
  const serverOptions: ServerOptions = {
    command: getSetting("jjpwrgem.path").trim() || "jjp",
    args: ["lsp"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "json" }],
  };

  client = new LanguageClient(
    "jjpwrgem",
    "jjpwrgem LSP",
    serverOptions,
    clientOptions,
  );

  context.subscriptions.push(client);
  return {
    whenReady: client.start().catch(() => {
      vscode.window
        .showErrorMessage(
          "Could not start the LSP server. Is `jjp` installed and on your PATH?",
          "Installation instructions",
        )
        .then((action) => {
          if (action === "Installation instructions") {
            void vscode.env.openExternal(
              vscode.Uri.parse(
                "https://github.com/20jasper/jjpwrgem#installation",
              ),
            );
          }
        });
    }),
  };
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
