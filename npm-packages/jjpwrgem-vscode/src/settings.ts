import * as vscode from "vscode";
import pkg from "../package.json";

type Properties = typeof pkg.contributes.configuration.properties;

/** Setting names declared in package.json */
export type SettingKey = keyof Properties;

/** Reads a setting, defaulting to the value in package.json */
export function getSetting<K extends SettingKey>(
  key: K,
): Properties[K]["default"] {
  return vscode.workspace
    .getConfiguration()
    .get(key, pkg.contributes.configuration.properties[key].default);
}
