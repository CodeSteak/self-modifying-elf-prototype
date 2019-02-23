extern crate ipc;
use ipc::{cbor, *};

fn main() {
    let ch: Channel<()> = Channel::new_from_env();
    let res: Option<cbor::Value> = ch.call(&(
        "core",
        "routes",
        "register",
        ("ls_route", "hello", "world!"),
    ));
    dbg!(res);
    let res: Option<Vec<Vec<cbor::Value>>> = ch.call(&["core", "routes", "list"]);
    for r in res.iter().flatten() {
        for i in r {
            match i {
                cbor::Value::String(s) => {
                    print!("/{}", s);
                }
                cbor::Value::Null => {
                    print!("/*");
                }
                a => {
                    print!("/{:?}", a);
                }
            }
        }
        println!();
    }
}
