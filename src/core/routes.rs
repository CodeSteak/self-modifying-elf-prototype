use crate::prelude::*;

#[service_state(("core", "routes", "register", Value::Null))]
fn register_fn(info: &mut PluginInfo, route: (Vec<Value>,)) -> Option<bool> {
    let mut rout = ROUTING_TABLE.lock().unwrap();

    rout.register(Box::new(RedirectHandler {
        path: route.0,
        info: info.clone(),
    }));

    Some(true)
}

#[service(("core", "routes", "list"))]
fn list_fn(_no_args: Vec<()>) -> Option<Vec<Vec<Value>>> {
    let rout = ROUTING_TABLE.lock().unwrap();

    let ret = rout
        .paths
        .iter()
        .map(|(r, _)| r.clone())
        .collect::<Vec<_>>();

    Some(ret)
}

struct RedirectHandler {
    path: Vec<Value>,
    info: PluginInfo,
}

impl<S: Send + Sized> Handler<S> for RedirectHandler {
    fn path(&self) -> Url {
        self.path.clone()
    }
    fn handle(&self, arg: &[&Value]) -> Option<Value> {
        let mut out = self.info.call_channel.as_ref().unwrap().lock().unwrap();

        let mut path_with_args: Vec<&Value> = vec![];

        let mut arg_iter = arg.iter();
        for p in self.path.iter() {
            if p.is_null() {
                path_with_args.push(
                    arg_iter
                        .next()
                        .expect("Routing Error. Should not have been routed here!"),
                );
            } else {
                path_with_args.push(p);
            }
        }

        assert_eq!(None, arg_iter.next());

        cbor::to_writer(&mut *out, &path_with_args).ok()?;
        cbor::from_reader::<Value, _>(&mut *out).ok()
    }
}
