const NODE_BUILTIN_MODULES: [&str; 53] = [
    "assert",
    "assert/strict",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "dns/promises",
    "domain",
    "events",
    "fs",
    "fs/promises",
    "http",
    "http2",
    "https",
    "inspector",
    "inspector/promises",
    "module",
    "net",
    "os",
    "path",
    "path/posix",
    "path/win32",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "readline/promises",
    "repl",
    "stream",
    "stream/consumers",
    "stream/promises",
    "stream/web",
    "string_decoder",
    "timers",
    "timers/promises",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "util/types",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImportKind {
    Builtin,
    External,
    Internal,
    Parent,
    Sibling,
    Index,
}

impl ImportKind {
    pub fn guess(name: &str) -> Self {
        if name == "bun" || name.starts_with("node:") {
            return Self::Builtin;
        }

        if NODE_BUILTIN_MODULES.contains(&name) {
            return Self::Builtin;
        }

        if name == "." || name == "./" || name == "./index" || name.starts_with("./index.") {
            return Self::Index;
        }

        if name.starts_with("./") {
            return Self::Sibling;
        }

        if name.starts_with("../") {
            return Self::Parent;
        }

        let mut chars = name.chars();
        loop {
            let Some(char) = chars.next() else {
                break;
            };

            if char == '@' {
                continue;
            }

            if char.is_ascii_alphabetic() {
                return Self::External;
            }

            break;
        }

        Self::Internal
    }
}
