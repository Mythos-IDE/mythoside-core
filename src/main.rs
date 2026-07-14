use mythoside_core::rpc::{dispatch, Notifier, Request, Response};
use mythoside_core::watcher::WatcherState;
use std::io::{self, BufRead};

/// The MythosIDE local core server. Speaks JSON-RPC-ish messages over
/// stdin/stdout — one request per line in, one response (or, for watcher
/// events, an unsolicited notification) per line out. Deliberately not an
/// HTTP server: stdio confines communication to this process's own parent
/// (the Tauri shell that spawns it), with no listening port for another
/// local process to probe — matching the product's local-first / "your
/// data never leaves this device" posture. See CLAUDE.md for the full
/// rationale and the client-side (src-tauri) half of this protocol.
fn main() {
    let watcher_state = WatcherState::default();
    let notifier = Notifier::default();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: Request = match serde_json::from_str(line) {
            Ok(request) => request,
            Err(err) => {
                eprintln!("mythoside-core: invalid request, skipping: {err}");
                continue;
            }
        };

        let response = match dispatch(&request.method, request.params, &watcher_state, &notifier) {
            Ok(result) => Response {
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(error) => Response {
                id: request.id,
                result: None,
                error: Some(error),
            },
        };

        notifier.send_response(&response);
    }
}
