use crate::prelude::*;
use crate::runner::PluginInfo;

#[service("core", "routes", "register")]
fn register_fn(ctx: &mut Context, route: Vec<Value>) -> bool {
    let mut rout = ctx.global_routes.lock().unwrap();

    rout.register(Box::new(RedirectHandler {
        path: route,
        info: ctx
            .plugin_info
            .clone()
            .expect("Only plugins can call this."),
    }));

    true
}

#[service("core", "routes", "list")]
fn list_fn(ctx: &mut Context) -> Vec<Vec<Value>> {
    let rout = ctx.global_routes.lock().unwrap();

    let ret = rout
        .paths
        .iter()
        .map(|(r, _)| r.clone())
        .collect::<Vec<_>>();

    ret
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
        let mut out = self.info.call_channel.as_ref().lock().unwrap();

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
