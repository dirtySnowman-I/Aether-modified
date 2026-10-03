//! Android binding for the exact TCP/native UDP relay shipped in GUI v0.9.2.
use aether_exit_proxy::{Config, Credentials, Kind, Server};
use jni::{JNIEnv, objects::{JClass, JString}, sys::{jboolean, jint, jstring}};
use serde::Deserialize;
use std::{io, net::{SocketAddr, TcpListener}, sync::{Arc, Mutex, OnceLock}};

struct Session { port: u16, server: Server }
fn session() -> &'static Mutex<Option<Session>> {
    static STATE: OnceLock<Mutex<Option<Session>>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(None))
}
fn error() -> &'static Mutex<String> {
    static ERROR: OnceLock<Mutex<String>> = OnceLock::new();
    ERROR.get_or_init(|| Mutex::new(String::new()))
}
fn set_error(s: impl ToString) { *error().lock().unwrap_or_else(|p| p.into_inner()) = s.to_string(); }
#[derive(Deserialize)]
struct Input {
    kind: String, host: String, port: u16, udp_enabled: bool,
    username: Option<String>, password: Option<String>,
}
fn config(text: &str) -> io::Result<Config> {
    let p: Input = serde_json::from_str(text).map_err(|_| io::Error::other("Invalid final proxy configuration"))?;
    let c = Config {
        kind: match p.kind.as_str() { "socks5" => Kind::Socks5, "http" => Kind::Http,
            _ => return Err(io::Error::other("Choose SOCKS5 or HTTP CONNECT")) },
        host: p.host, port: p.port, udp_enabled: p.udp_enabled,
        credentials: match (p.username, p.password) {
            (None, None) => None,
            (Some(username), Some(password)) => Some(Credentials { username, password }),
            _ => return Err(io::Error::other("Enter both username and password")),
        },
    };
    c.validate()?; Ok(c)
}

#[no_mangle]
pub extern "system" fn Java_studio_cluvex_aether_core_FinalProxy_nativeStart(
    mut env: JNIEnv, _: JClass, json: JString, core_port: jint,
) -> jint {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> io::Result<u16> {
        if !(1..=65535).contains(&core_port) { return Err(io::Error::other("Invalid Aether port")); }
        let text: String = env.get_string(&json).map_err(|_| io::Error::other("Cannot read proxy settings"))?.into();
        let config = config(&text)?;
        let mut guard = session().lock().unwrap_or_else(|p| p.into_inner());
        if guard.is_some() { return Err(io::Error::other("A final proxy session is already active")); }
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let core: SocketAddr = format!("127.0.0.1:{core_port}").parse().unwrap();
        let server = Server::start(listener, core, config, Arc::new(set_error))?;
        *guard = Some(Session { port, server });
        set_error("");
        Ok(port)
    }));
    match result { Ok(Ok(port)) => port as jint, Ok(Err(e)) => { set_error(e); 0 },
        Err(_) => { set_error("Final proxy engine stopped unexpectedly"); 0 } }
}

#[no_mangle]
pub extern "system" fn Java_studio_cluvex_aether_core_FinalProxy_nativeVerify(
    _: JNIEnv, _: JClass, port: jint,
) -> jboolean {
    let result = std::panic::catch_unwind(|| -> io::Result<()> {
        let probe = {
            let guard = session().lock().unwrap_or_else(|p| p.into_inner());
            let s = guard.as_ref().filter(|s| i32::from(s.port) == port)
                .ok_or_else(|| io::Error::other("Final proxy session was cancelled"))?;
            s.server.probe()
        };
        // Never hold the state lock across network I/O. nativeStop can close
        // pending TCP/UDP handshakes immediately when the user disconnects.
        probe.verify()
    });
    match result { Ok(Ok(())) => 1, Ok(Err(e)) => { set_error(e); 0 },
        Err(_) => { set_error("Final proxy check stopped unexpectedly"); 0 } }
}

#[no_mangle]
pub extern "system" fn Java_studio_cluvex_aether_core_FinalProxy_nativeStop(
    _: JNIEnv, _: JClass, port: jint,
) {
    let _ = std::panic::catch_unwind(|| {
        let removed = {
            let mut guard = session().lock().unwrap_or_else(|p| p.into_inner());
            if guard.as_ref().is_some_and(|s| i32::from(s.port) == port) { guard.take() } else { None }
        };
        drop(removed); // closes every TCP connection and native UDP association
    });
}

#[no_mangle]
pub extern "system" fn Java_studio_cluvex_aether_core_FinalProxy_nativeError(
    env: JNIEnv, _: JClass,
) -> jstring {
    let s = error().lock().unwrap_or_else(|p| p.into_inner()).clone();
    env.new_string(s).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rejects_http_udp_and_bad_credentials() {
        assert!(config(r#"{"kind":"http","host":"proxy.test","port":1080,"udp_enabled":true}"#).is_err());
        assert!(config(r#"{"kind":"socks5","host":"proxy.test","port":1080,"udp_enabled":true,"username":"u","password":""}"#).is_err());
        let c = config(r#"{"kind":"socks5","host":"proxy.test","port":1080,"udp_enabled":true,"username":"u","password":"secret"}"#).unwrap();
        assert!(!format!("{c:?}").contains("secret"));
    }
}

