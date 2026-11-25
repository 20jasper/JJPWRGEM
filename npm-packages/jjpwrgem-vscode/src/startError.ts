export const INSTALL_ACTION = "Installation instructions";
export const SETTINGS_ACTION = "Open settings";

export function describeStartError(
  command: string,
  custom: boolean,
  error: unknown,
): { message: string; actions: string[] } {
  const reason = error instanceof Error ? error.message : String(error);
  if (custom) {
    return {
      message: `Could not start the jjpwrgem language server with "${command}": ${reason}. Check the jjpwrgem.path setting.`,
      actions: [SETTINGS_ACTION],
    };
  }
  return {
    message: `Could not start the jjpwrgem language server: ${reason}. Is jjp installed and on your PATH?`,
    actions: [INSTALL_ACTION],
  };
}
