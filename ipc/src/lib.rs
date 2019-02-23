extern crate bufstream;
extern crate serde;
extern crate serde_cbor;

pub mod cbor {
    pub use serde_cbor::*;
    // from_reader does a eof check. This is bad!.
    pub fn from_stream_reader<T, R>(reader: R) -> serde_cbor::error::Result<T>
    where
        T: serde::de::DeserializeOwned,
        R: std::io::Read,
    {
        let mut deserializer = serde_cbor::Deserializer::from_reader(reader);
        let value = serde::de::Deserialize::deserialize(&mut deserializer)?;
        Ok(value)
    }
}

pub mod bs {
    pub use bufstream::*;
}

use bufstream::BufStream;

use serde::{de::DeserializeOwned, Serialize};

use std::io::prelude::*;

use std::os::unix::net::UnixStream;
use std::os::unix::prelude::*;

use std::sync::*;
use std::thread;

#[derive(Clone)]
pub struct Channel<State: Send + Sized + 'static = ()> {
    outgoing: Arc<Mutex<BufStream<UnixStream>>>,
    routes: Arc<Mutex<Router<State>>>,
}

impl<State: Send + Sized + 'static> Channel<State> {
    pub fn new((i, o): (UnixStream, UnixStream)) -> Self
    where
        State: Default,
    {
        let o = Arc::new(Mutex::new(BufStream::new(o)));
        Self::new_full((i, o), Default::default(), Default::default())
    }

    pub fn new_full(
        (incoming, outgoing): (UnixStream, Arc<Mutex<BufStream<UnixStream>>>),
        routes: Arc<Mutex<Router<State>>>,
        mut state: State,
    ) -> Self {
        {
            let routes = routes.clone();
            let mut incoming = incoming;
            thread::spawn(move || {
                while let Ok(data) = cbor::from_stream_reader::<serde_cbor::Value, _>(&mut incoming)
                {
                    if let Some(url) = data.as_array() {
                        let handler = { routes.lock().unwrap().get_handler(url) };

                        if let Some((handler, arg)) = handler {
                            let res = handler.lock().unwrap().handle_with_state(&mut state, &arg);
                            serde_cbor::to_writer(&mut incoming, &res).ok()?;
                        } else {
                            serde_cbor::to_writer(&mut incoming, &(None as Option<()>)).ok()?;
                        }
                    }
                    incoming.flush().ok()?;
                }
                Some(())
            });
        }

        Self { outgoing, routes }
    }

    pub fn new_from_env() -> Self
    where
        State: Default,
    {
        let h2p_fd = std::env::var("HostToPlugin_FD")
            .expect("Expected HostToPlugin_FD Environment Var")
            .parse::<RawFd>()
            .expect("Expected HostToPlugin_FD to be an int.");

        let p2h_fd = std::env::var("PluginToHost_FD")
            .expect("Expected PluginToHost_FD Environment Var")
            .parse::<RawFd>()
            .expect("Expected PluginToHost_FD to be an int.");

        let h2p_sock = unsafe { UnixStream::from_raw_fd(h2p_fd) };
        let p2h_sock = unsafe { UnixStream::from_raw_fd(p2h_fd) };

        Self::new((h2p_sock, p2h_sock))
    }

    pub fn new_from_fd_0_1() -> Self
    where
        State: Default,
    {
        let h2p_sock = unsafe { UnixStream::from_raw_fd(0) };
        let p2h_sock = unsafe { UnixStream::from_raw_fd(1) };

        Self::new((h2p_sock, p2h_sock))
    }

    pub fn register(&mut self, handler: Box<dyn Handler<State>>) -> bool {
        let res: bool = self
            .call(("core", "routes", "register", &handler.path()))
            .unwrap_or(false);

        if res {
            self.routes.lock().unwrap().register(handler);
        }

        res
    }

    pub fn call<S: Serialize, R: DeserializeOwned>(&self, data: S) -> Option<R> {
        let mut out = self.outgoing.lock().unwrap();

        let _ = serde_cbor::to_writer(&mut *out, &data).ok()?;

        out.flush().ok()?;
        cbor::from_stream_reader::<Option<R>, _>(&mut *out).ok()?
    }
}

#[derive(Default)]
pub struct Router<State: Send + Sized = ()> {
    pub paths: Vec<(Url, Arc<Mutex<Box<dyn Handler<State>>>>)>,
}

impl<State: Send + Sized> Router<State> {
    pub fn register(&mut self, handler: Box<dyn Handler<State>>) {
        let path = handler.path();

        let filtered = self.paths.drain(0..).filter(|(p, _)| p != &path).collect();

        self.paths = filtered;

        self.paths.push((path, Arc::new(Mutex::new(handler))));
    }

    fn get_handler<'a>(
        &self,
        data: &'a [serde_cbor::Value],
    ) -> Option<(
        Arc<Mutex<Box<dyn Handler<State>>>>,
        Vec<&'a serde_cbor::Value>,
    )> {
        for (route, handler) in self.paths.iter() {
            if let Some(matched) = Self::check(route, data) {
                return Some((Arc::clone(handler), matched));
            }
        }
        return None;
    }

    /*fn handle(&self, data : &[serde_cbor::Value]) -> Option<serde_cbor::Value> where State : Default {
        self.handle_with_state(&mut Default::default(), data)
    }

    // TODO use get_handler, than prevent deadlock.
    fn handle_with_state(&self,  state : &mut State, data : &[serde_cbor::Value]) -> Option<serde_cbor::Value> {
        for (route,handler) in self.paths.iter() {
            if let Some(matched) = Self::check(route, data) {
                return handler.handle_with_state(state, &matched);
            }
        }
        return None;
    }*/

    fn check<'a>(
        path: &Vec<serde_cbor::Value>,
        data: &'a [serde_cbor::Value],
    ) -> Option<Vec<&'a serde_cbor::Value>> {
        if path.len() != data.len() {
            return None;
        }

        let mut res = vec![];
        for (p, d) in path.iter().zip(data) {
            if !p.is_null() && p != d {
                return None;
            }

            if p.is_null() {
                res.push(d)
            }
        }

        Some(res)
    }
}

pub type Url = Vec<serde_cbor::Value>;

pub trait Handler<State: Send + Sized = ()>: Send {
    fn path(&self) -> Url;
    fn handle_with_state(
        &self,
        _: &mut State,
        arg: &[&serde_cbor::Value],
    ) -> Option<serde_cbor::Value> {
        self.handle(arg)
    }
    fn handle(&self, arg: &[&serde_cbor::Value]) -> Option<serde_cbor::Value>;
}

pub struct FnHandler<S: Serialize, D: Serialize + DeserializeOwned, R: Serialize + Send>(
    pub R,
    pub fn(D) -> Option<S>,
);

impl<S: Serialize, D: Serialize + DeserializeOwned, R: Serialize + Send, State: Send + Sized>
    Handler<State> for FnHandler<S, D, R>
{
    fn path(&self) -> Url {
        let r = serde_cbor::to_value(&self.0).unwrap();
        if let serde_cbor::Value::Array(arr) = r {
            arr
        } else {
            panic!("Invalid Path Desc.")
        }
    }

    fn handle(&self, arg: &[&serde_cbor::Value]) -> Option<serde_cbor::Value> {
        // This is bad.
        // This should be changed when serde_cbor/0.9.0/src/serde_cbor/value/value.rs line: 637-647
        // changes.
        let buf = serde_cbor::to_vec(&arg).unwrap();
        let d: D = serde_cbor::from_slice::<_>(&buf[..]).ok()?;

        let s = self.1(d)?;

        serde_cbor::to_value(s).ok()
    }
}

pub struct FnHandlerState<
    S: Default + Serialize,
    D: Default + Serialize + DeserializeOwned,
    R: Serialize + Send,
    State: Send + Sized,
>(pub R, pub fn(&mut State, D) -> Option<S>);
impl<
        S: Default + Serialize,
        D: Default + Serialize + DeserializeOwned,
        R: Serialize + Send,
        State: Send + Sized,
    > Handler<State> for FnHandlerState<S, D, R, State>
{
    fn path(&self) -> Url {
        let r = serde_cbor::to_value(&self.0).unwrap();
        if let serde_cbor::Value::Array(arr) = r {
            arr
        } else {
            panic!("Invalid Path Desc.")
        }
    }

    fn handle_with_state(
        &self,
        state: &mut State,
        arg: &[&serde_cbor::Value],
    ) -> Option<serde_cbor::Value> {
        // This is bad.
        // This should be changed when serde_cbor/0.9.0/src/serde_cbor/value/value.rs line: 637-647
        // changes.
        let buf = serde_cbor::to_vec(&arg).unwrap();
        let d: D = serde_cbor::from_slice::<_>(&buf[..]).ok()?;

        let s = self.1(state, d)?;

        serde_cbor::to_value(s).ok()
    }

    fn handle(&self, _arg: &[&serde_cbor::Value]) -> Option<serde_cbor::Value> {
        unreachable!()
    }
}
