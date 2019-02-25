pub mod rw;
pub use rw::*;

pub mod core {
    pub mod entry {
        use crate::*;
        use ipc::Channel;

        /* "core" "entry" "list" :
                       -> Vec < Entry >
        */
        pub fn list<S: Send + Sized + 'static>(ctx: &Channel<S>) -> Vec<Entry> {
            ctx.c(("core", "entry", "list"))
        }
        /* "core" "entry" "query" : QueryOperation
                                -> Vec < Entry >
        */
        pub fn query<S: Send + Sized + 'static>(
            ctx: &Channel<S>,
            q: &QueryOperation,
        ) -> Vec<Entry> {
            ctx.c(("core", "entry", "query", q))
        }
        /* "core" "entry" "read" : String
                                -> Option < Entry >
        */
        pub fn read<S: Send + Sized + 'static>(ctx: &Channel<S>, s: &str) -> Option<Entry> {
            ctx.c(("core", "entry", "read", s))
        }

        /* "core" "entry" "write" : WriteOperation
                        -> bool
        */
        pub fn write<S: Send + Sized + 'static>(ctx: &Channel<S>, w: &WriteOperation) -> bool {
            ctx.c(("core", "entry", "write", w))
        }
    }

    pub mod own_executable {
        use crate::*;
        use ipc::Channel;
        /* "core" "own_executable" "read" :
                        -> std :: result :: Result < ByteBuf , String >
        */
        pub fn read<S: Send + Sized + 'static>(
            ctx: &Channel<S>,
        ) -> std::result::Result<ByteBuf, String> {
            ctx.c(("core", "own_executable", "read"))
        }
    }

    pub mod routes {
        use ipc::cbor::Value;
        use ipc::Channel;

        /* "core" "routes" "register" : Vec < Value >
                                -> bool
        */
        pub fn register<S: Send + Sized + 'static>(ctx: &Channel<S>, pattern: &Vec<Value>) -> bool {
            ctx.c(("core", "routes", "register", pattern))
        }
        /* "core" "routes" "list" :
                                -> Vec < Vec < Value > >
        */
        pub fn list<S: Send + Sized + 'static>(ctx: &Channel<S>) -> Vec<Vec<Value>> {
            ctx.c(("core", "routes", "list"))
        }
    }

    pub mod hash {
        use crate::*;
        use ipc::Channel;

        /* "core" "hash" "list" :
                                -> Vec < HashRef >
        */
        pub fn list<S: Send + Sized + 'static>(ctx: &Channel<S>) -> Vec<HashRef> {
            ctx.c(("core", "hash", "list"))
        }
        /* "core" "hash" "read" : HashRef
                                -> Option < Arc < ByteBuf > >
        */
        pub fn read<S: Send + Sized + 'static>(
            ctx: &Channel<S>,
            hash: &HashRef,
        ) -> Option<ByteBuf> {
            ctx.c(("core", "hash", "read", hash))
        }
    }
}
