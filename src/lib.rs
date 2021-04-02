extern crate libc;
extern crate zmq;
extern crate dupl_server_proto;

use std::{ffi, mem, ptr, slice};
use std::borrow::Cow;
use libc::{c_int, c_void, c_char, c_ulong, size_t};
use dupl_server_proto as proto;
use dupl_server_proto::bin::{ToBin, FromBin};

pub enum ErrorMsg {
    Native(Cow<'static, str>),
    Foreign(ffi::CString),
    Invalid,
}

impl ErrorMsg {
    fn from_str(message: &'static str) -> ErrorMsg {
        ErrorMsg::Native(Cow::Borrowed(message))
    }

    fn from_string(message: String) -> ErrorMsg {
        ErrorMsg::Native(Cow::Owned(message))
    }
}

pub struct Config {
    connect_addr: String,
    timeout_ms: u64,
    context: zmq::Context,
}

pub struct DuplClient {
    sock: Option<zmq::Socket>,
    config: Option<Config>,
    reply: Option<String>,
    last_error: Option<ErrorMsg>,
}

#[no_mangle]
pub extern fn dupl_client_create(dcp: *mut *mut DuplClient) -> c_int {
    if dcp.is_null() {
        return 1;
    }

    let area = unsafe { libc::malloc(mem::size_of::<DuplClient>() as size_t) };
    if area.is_null() {
        return 1;
    }

    let client = DuplClient {
        sock: None,
        config: None,
        reply: None,
        last_error: None,
    };
    let client_area = area as *mut DuplClient;
    unsafe {
        ptr::write(client_area, client);
        *dcp = client_area;
    }

    0
}

#[no_mangle]
pub extern fn dupl_client_close(dcp: *mut *mut DuplClient) -> c_int {
    if dcp.is_null() {
        return 1;
    }

    match unsafe { *dcp } {
        dc if dc.is_null() => 0,
        dc => {
            let _client = unsafe { ptr::read(dc) };
            unsafe {
                libc::free(dc as *mut c_void);
                *dcp = ptr::null_mut();
            }
            0
        }
    }
}

#[no_mangle]
pub extern fn dupl_client_init(dc: *mut DuplClient, zmq_connect_addr: *const c_char, req_timeout_ms: c_ulong) -> c_int {
    if dc.is_null() {
        return 1;
    }

    let client = unsafe { &mut *dc };
    let result = match client.config {
        Some(..) =>
            Err(ErrorMsg::from_str("already initialized")),
        None =>
            match String::from_utf8(unsafe { ffi::CStr::from_ptr(zmq_connect_addr).to_bytes().iter().cloned().collect() }) {
                Ok(ref zmq_addr) if zmq_addr.len() == 0 =>
                    Err(ErrorMsg::from_str("invalid zero length zmq address")),
                Ok(valid_zmq_addr) =>
                    Ok(Config {
                        connect_addr: valid_zmq_addr,
                        timeout_ms: req_timeout_ms as u64,
                        context: zmq::Context::new(),
                    }),
                Err(error) =>
                    Err(ErrorMsg::from_string(format!("invalid zmq address: {}", error))),
            },
    };

    match result {
        Ok(config) => { client.config = Some(config); 0 },
        Err(msg) => { client.last_error = Some(msg); 1 },
    }
}

enum RequestError {
    TimedOut,
    Error(ErrorMsg),
}

impl From<ErrorMsg> for RequestError {
    fn from(err: ErrorMsg) -> RequestError {
        RequestError::Error(err)
    }
}

#[no_mangle]
pub extern fn dupl_client_request(
    dc: *mut DuplClient,
    req_json: *const c_char,
    req_json_length: size_t,
    rep_json: *mut *const c_char,
    rep_json_length: *mut size_t,
    pretty_print: c_int) -> c_int
{
    if dc.is_null() || req_json.is_null() {
        return 1;
    }

    let client = unsafe { &mut *dc };
    let req_json_slice = unsafe { slice::from_raw_parts(req_json as *const u8, req_json_length as usize) };

    match native_request(client, req_json_slice, pretty_print != 0) {
        Ok(reply) => {
            if !rep_json.is_null() {
                unsafe { *rep_json = reply.as_ptr() as *const c_char }
            }
            if !rep_json_length.is_null() {
                unsafe { *rep_json_length = reply.as_bytes().len() as size_t }
            }
            client.reply = Some(reply);
            0
        },
        Err(RequestError::TimedOut) => {
            client.last_error = Some(ErrorMsg::from_str("timed out"));
            client.sock = None;
            -1
        },
        Err(RequestError::Error(msg)) => {
            client.last_error = Some(msg);
            1
        },
    }
}

fn native_request(client: &mut DuplClient, req_json_slice: &[u8], pretty_print: bool) -> Result<String, RequestError> {
    loop {
        if let Some(ref mut sock) = client.sock {
            let req_str = std::str::from_utf8(req_json_slice)
                .map_err(|e| ErrorMsg::from_string(format!("request utf8 error: {}", e)))?;
            let valid_json_string: String = req_str.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
            let req: proto::Trans<String> = proto::json::json_str_to_anything(&valid_json_string)
                .map_err(|e| {
                    ErrorMsg::from_string(format!("request parse failed: {}", e))
                })?;

            let required = req.encode_len();
            let mut req_msg =
                zmq::Message::with_size(required);
            req.encode(&mut req_msg);
            sock.send(req_msg, 0)
                .map_err(|e| ErrorMsg::from_string(format!("zmq send failed: {}", e)))?;

            let sock_online = {
                let mut pollitems = [sock.as_poll_item(zmq::POLLIN)];
                zmq::poll(&mut pollitems, client.config.as_ref().map(|cfg| cfg.timeout_ms as i64).unwrap_or(-1))
                    .map_err(|e| {
                        ErrorMsg::from_string(format!("zmq poll failed: {}", e))
                    })?;
                pollitems[0].get_revents() == zmq::POLLIN
            };
            if sock_online {
                let rep_msg = sock.recv_msg(0)
                    .map_err(|e| ErrorMsg::from_string(format!("zmq recv failed: {}", e)))?;
                let (rep, _) = proto::Rep::<String>::decode(&rep_msg)
                    .map_err(|e| {
                        ErrorMsg::from_string(format!("rep packet decoding fail: {}", e))
                    })?;
                let rep_json = proto::json::rep_to_json(&rep);
                return Ok(if pretty_print { format!("{}", rep_json.pretty()) } else { format!("{}", rep_json) })
            } else {
                return Err(RequestError::TimedOut)
            }
        } else if let Some(ref mut cfg) = client.config {
            let socket = cfg.context
                .socket(zmq::REQ)
                .map_err(|e| ErrorMsg::from_string(format!("zmq socket failed: {}", e)))?;
            socket.set_linger(0)
                .map_err(|e| ErrorMsg::from_string(format!("zmq zero linger failed: {}", e)))?;
            socket.connect(&cfg.connect_addr[..])
                .map_err(|e| {
                    ErrorMsg::from_string(format!("zmq connect to {} failed: {}", cfg.connect_addr, e))
                })?;
            client.sock = Some(socket);
        }
    }
}

#[no_mangle]
pub extern fn dupl_client_last_error(dc: *mut DuplClient) -> *const c_char {
    if dc.is_null() {
        return ptr::null();
    }

    let client = unsafe { &mut *dc };
    match client.last_error {
        None =>
            ptr::null(),
        Some(ErrorMsg::Invalid) =>
            b"error message cannot be displayed\0".as_ptr() as *const c_char,
        Some(ErrorMsg::Foreign(ref msg)) =>
            msg.as_ptr(),
        Some(ref mut native_msg) => {
            if let ErrorMsg::Native(ref msg) = mem::replace(native_msg, ErrorMsg::Invalid) {
                if let Ok(foreign_msg) = ffi::CString::new(msg.as_bytes()) {
                    let _ = mem::replace(native_msg, ErrorMsg::Foreign(foreign_msg));
                }
            }

            dupl_client_last_error(dc)
        }
    }
}

#[cfg(test)]
mod test {
    use std::{ptr, ffi, str, slice, thread};
    use std::sync::mpsc::{sync_channel, TryRecvError};
    use libc::{c_char, size_t};
    use dupl_server_proto as proto;
    use dupl_server_proto::bin::{ToBin, FromBin};
    use zmq;
    use super::DuplClient;
    use super::{dupl_client_create, dupl_client_close, dupl_client_init, dupl_client_request, dupl_client_last_error};

    #[test]
    fn create_close() {
        let mut client: *mut DuplClient = ptr::null_mut();
        assert_eq!(dupl_client_create(&mut client), 0);
        assert!(!client.is_null());
        assert_eq!(dupl_client_close(&mut client), 0);
        assert!(client.is_null());
    }

    macro_rules! try_call {
        ($func:ident, $client:expr, $($arg:expr),*) => ({
            let client = $client;
            if $func(client, $($arg),*) != 0 {
                let ffi_msg = dupl_client_last_error(client);
                let msg = String::from_utf8(unsafe { ffi::CStr::from_ptr(ffi_msg).to_bytes().iter().cloned().collect() }).unwrap();
                panic!("last error: {}", msg)
            }
        })
    }

    #[test]
    fn init() {
        let mut client: *mut DuplClient = ptr::null_mut();
        assert_eq!(dupl_client_create(&mut client), 0);
        assert!(!client.is_null());
        let ffi_zmq_addr = ffi::CString::new("ipc:///tmp/sock_a".as_bytes()).unwrap();
        try_call!(dupl_client_init, client, ffi_zmq_addr.as_ptr(), 1000);
        assert_eq!(dupl_client_close(&mut client), 0);
        assert!(client.is_null());
    }

    #[test]
    fn request() {
        let zmq_addr = "ipc:///tmp/sock_b";
        // server
        let ctx = zmq::Context::new();
        let sock = ctx.socket(zmq::REP).unwrap();
        sock.set_linger(0).unwrap();
        sock.bind(zmq_addr).unwrap();
        let (server_tx, server_rx) = sync_channel(0);
        let server_thread = thread::spawn(move || {
            let req_msg = sock.recv_msg(0).unwrap();
            let (trans, _) = proto::Trans::<String>::decode(&req_msg).unwrap();
            let rep = proto::Rep::Unexpected(match trans {
                proto::Trans::Async(req) => req,
                proto::Trans::Sync(req) => req,
            });
            let required = rep.encode_len();
            let mut rep_msg = zmq::Message::with_capacity(required);
            rep.encode(&mut rep_msg);
            sock.send(rep_msg, 0).unwrap();
            server_tx.send(()).unwrap();
        });
        // client
        let (client_tx, client_rx) = sync_channel(0);
        let client_thread = thread::spawn(move || {
            let mut client: *mut DuplClient = ptr::null_mut();
            assert_eq!(dupl_client_create(&mut client), 0);
            assert!(!client.is_null());
            let ffi_zmq_addr = ffi::CString::new(zmq_addr.as_bytes()).unwrap();
            try_call!(dupl_client_init, client, ffi_zmq_addr.as_ptr(), 1000);
            let req_json = proto::json::req_to_json(&proto::Trans::Async(proto::Req::Lookup(proto::Workload::Single(proto::LookupTask {
                text: "some text to lookup".to_owned(),
                result: proto::LookupType::BestOrMine,
                post_action: proto::PostAction::InsertNew {
                    cond: proto::InsertCond::BestSimLessThan(0.5),
                    assign: proto::ClusterAssign {
                        cond: proto::AssignCond::Always,
                        choice: proto::ClusterChoice::ClientChoice(177),
                    },
                    user_data: "some user data".to_owned(),
                }
            }))));
            let req_json_string = format!("{}", req_json);
            let ffi_req_json_string = req_json_string.as_bytes().as_ptr() as *const c_char;
            let ffi_req_json_string_len = req_json_string.as_bytes().len() as size_t;
            let mut rep_json: *const c_char = ptr::null();
            let mut rep_json_length: size_t = 0;
            try_call!(dupl_client_request, client, ffi_req_json_string, ffi_req_json_string_len, &mut rep_json, &mut rep_json_length, 0);
            assert!(!rep_json.is_null());
            assert!(rep_json_length > 0);
            let rep_json_slice = unsafe { slice::from_raw_parts(rep_json as *const u8, rep_json_length as usize) };
            let rep_str = str::from_utf8(rep_json_slice).unwrap();
            let rep: proto::Rep<String> = proto::json::json_str_to_anything(&rep_str).unwrap();
            match rep {
                proto::Rep::Unexpected(proto::Req::Lookup(proto::Workload::Single(proto::LookupTask {
                    text: ref rep_text,
                    result: proto::LookupType::BestOrMine,
                    post_action: proto::PostAction::InsertNew {
                        cond: proto::InsertCond::BestSimLessThan(0.5),
                        assign: proto::ClusterAssign {
                            cond: proto::AssignCond::Always,
                            choice: proto::ClusterChoice::ClientChoice(177),
                        },
                        user_data: ref rep_user_data,
                    }
                }))) if rep_text == "some text to lookup" && rep_user_data == "some user data" => (),
                other => panic!("unexpected rep: {:?}", other),
            }
            assert_eq!(dupl_client_close(&mut client), 0);
            assert!(client.is_null());
            client_tx.send(()).unwrap();
        });
        // master
        let (mut server_finished, mut client_finished) = (false, false);
        while !server_finished && !client_finished {
            if !server_finished {
                match server_rx.try_recv() {
                    Ok(()) => server_finished = true,
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => panic!("server thread is down"),
                }
            }
            if !client_finished {
                match client_rx.try_recv() {
                    Ok(()) => client_finished = true,
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => panic!("client thread is down"),
                }
            }
            thread::sleep(::std::time::Duration::from_millis(100));
        }
        client_thread.join().unwrap();
        server_thread.join().unwrap();
    }
}
