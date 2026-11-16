import * as vscode from "vscode";
import { getSetting } from "./settings";
import {
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions,
} from "vscode-languageclient/node";

const MIN_JJP_VERSION = "0.7.7";
const INSTALLATION_URL = "https://github.com/20jasper/jjpwrgem#installation";
const INSTALLATION_ACTION = "Update jjp";

let client: LanguageClient;

function versionParts(version: string): number[] {
  return version.split(".").map((part) => Number.parseInt(part, 10) || 0);
}

function isOlderVersion(version: string, minimum: string): boolean {
  const found = versionParts(version);
  const required = versionParts(minimum);
  for (let index = 0; index < 3; index++) {
    const difference = (found[index] ?? 0) - (required[index] ?? 0);
    if (difference !== 0) {
      return difference < 0;
    }
  }
  return false;
}

function openInstallation(): void {
  void vscode.env.openExternal(vscode.Uri.parse(INSTALLATION_URL));
}

function warnWhenServerIsTooOld(): void {
  const version = client.initializeResult?.serverInfo?.version;
  if (version !== undefined && !isOlderVersion(version, MIN_JJP_VERSION)) {
    return;
  }
  void vscode.window
    .showWarningMessage(
      `jjp ${MIN_JJP_VERSION} or newer is needed for jjpwrgem.format settings.`,
      INSTALLATION_ACTION,
    )
    .then((action) => {
      if (action === INSTALLATION_ACTION) {
        openInstallation();
      }
    });
}

export function activate(context: vscode.ExtensionContext): {
  whenReady: Promise<void>;
} {
  const serverOptions: ServerOptions = {
    command: getSetting("jjpwrgem.path").trim() || "jjp",
    args: ["lsp"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "json" }],
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
  return {
    whenReady: client
      .start()
      .then(warnWhenServerIsTooOld)
      .catch(() => {
        vscode.window
          .showErrorMessage(
            "Could not start the LSP server. Is `jjp` installed and on your PATH?",
            "Installation instructions",
          )
          .then((action) => {
            if (action === "Installation instructions") {
              openInstallation();
            }
          });
      }),
  };
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
