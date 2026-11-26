use jjpwrgem_parse::settings::{Setting, choice, choice_enum, settings, string};
use serde_json::{Map, Value};

use crate::vscode_settings::schema;

const RELOAD_SENTENCE: &str = "Reload the window after changing.";

#[derive(Clone, Copy)]
enum Scope {
    Machine,
    Window,
}

impl Scope {
    const fn name(self) -> &'static str {
        match self {
            Self::Machine => "machine",
            Self::Window => "window",
        }
    }
}

struct VscodeMeta {
    scope: Scope,
    reload: bool,
}

choice_enum! {
    pub enum TraceServer {
        /// No tracing
        #[default]
        Off = "off",
        /// Trace requests and notifications
        Messages = "messages",
        /// Trace requests and notifications with their payloads
        Verbose = "verbose",
    }
}

settings! {
    struct ExtensionSettings, id ExtensionSetting {
        /// Path to the `jjp` executable used to run the language server. Defaults to `jjp` on your `PATH`
        Path path: Option<String> = string("jjp"),
    }
}

settings! {
    struct TraceSettings, id TraceSetting {
        /// Traces the communication between VS Code and the jjpwrgem language server
        Server server: TraceServer = choice(),
    }
}

const fn extension_meta(id: ExtensionSetting) -> VscodeMeta {
    match id {
        ExtensionSetting::Path => VscodeMeta {
            scope: Scope::Machine,
            reload: true,
        },
    }
}

const fn trace_meta(id: TraceSetting) -> VscodeMeta {
    match id {
        TraceSetting::Server => VscodeMeta {
            scope: Scope::Window,
            reload: false,
        },
    }
}

fn property(setting: &Setting, meta: &VscodeMeta) -> Value {
    let mut schema = schema(setting);
    schema["scope"] = meta.scope.name().into();
    let description = format!("{}.", setting.description);
    schema["markdownDescription"] = if meta.reload {
        format!("{description} {RELOAD_SENTENCE}")
    } else {
        description
    }
    .into();
    schema
}

/// Entries for the extension's own `jjpwrgem.*` keys, then its `jjpwrgem.trace.*` keys.
pub fn extension_properties() -> (Map<String, Value>, Map<String, Value>) {
    let extension = ExtensionSetting::ALL
        .iter()
        .map(|&id| {
            let setting = id.setting();
            (
                format!("jjpwrgem.{}", setting.key()),
                property(setting, &extension_meta(id)),
            )
        })
        .collect();
    let trace = TraceSetting::ALL
        .iter()
        .map(|&id| {
            let setting = id.setting();
            (
                format!("jjpwrgem.trace.{}", setting.key()),
                property(setting, &trace_meta(id)),
            )
        })
        .collect();
    (extension, trace)
}
