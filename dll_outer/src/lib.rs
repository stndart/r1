use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

use std::sync::{Condvar, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tonic::Request;
use tonic::transport::Channel;

use one::AddRequest;
use one::one_client::OneClient;

pub mod one {
    tonic::include_proto!("one");
}

static RUNTIME_HANDLE: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);
static mut CLIENT: Option<OneClient<Channel>> = None;

static SHUTDOWN_TX: Mutex<Option<oneshot::Sender<()>>> = Mutex::new(None);
static INIT_CONDVAR: Condvar = Condvar::new();
static INIT_STATE: Mutex<bool> = Mutex::new(false);

#[unsafe(no_mangle)]
pub extern "C" fn add(left: i64, right: i64) -> i64 {
    expect_initialized();

    Runtime::new().unwrap().block_on(add_impl(left, right))
}

async fn add_impl(left: i64, right: i64) -> i64 {
    let request = Request::new(AddRequest { a: left, b: right });
    unsafe { (*std::ptr::addr_of_mut!(CLIENT)).as_mut().unwrap() }
        .add(request)
        .await
        .unwrap()
        .into_inner()
        .result
}

fn expect_initialized() {
    let mut state = INIT_STATE.lock().unwrap();
    while !*state {
        state = INIT_CONDVAR.wait(state).unwrap();
    }
}

fn start_runtime() {
    let handle = thread::spawn(|| {
        let runtime = Runtime::new().unwrap();
        runtime.block_on(async {
            let (tx, rx) = oneshot::channel();
            *SHUTDOWN_TX.lock().unwrap() = Some(tx);

            let client = OneClient::connect("http://127.0.0.1:50051").await;

            let mut success = false;
            match client {
                Ok(client) => {
                    unsafe { CLIENT = Some(client) };
                    success = true;
                }
                Err(e) => {
                    println!("Error connecting to server: {e}");
                }
            }

            {
                let mut state = INIT_STATE.lock().unwrap();
                *state = true;
                INIT_CONDVAR.notify_all();
            }

            if success {
                println!("Runtime is ready");
                let _ = rx.await;
            }

            println!("Shutting down runtime");
        });
    });
    *RUNTIME_HANDLE.lock().unwrap() = Some(handle);
}

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => start_runtime(),
        DLL_PROCESS_DETACH => {
            if let Some(tx) = SHUTDOWN_TX.lock().unwrap().take() {
                let _ = tx.send(());
            }
            // CRITICAL: We cannot join the thread in DllMain because it will either deadlock
            // on the loader lock or panic if the OS has already terminated the thread during process exit.
            // We just send the shutdown signal and let the OS tear down the thread.
            if let Some(handle) = RUNTIME_HANDLE.lock().unwrap().take() {
                // handle.join().unwrap(); // DO NOT DO THIS
            }
        }
        _ => (),
    }
    true
}
