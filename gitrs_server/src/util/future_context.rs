use futures::{future, Future};
use std;
use std::sync::{Arc, Mutex};

// See https://github.com/rust-lang/rfcs/issues/2407#issuecomment-385291238.
macro_rules! enclose {
    (($( $x:ident ),*) $y:expr) => {
        {
            $(let $x = $x.clone();)*
                $y
        }
    };
}

pub fn inject<Context: 'static + Send, Item: 'static, Error: 'static>(
    context: Context,
    future: Box<Future<Item = Item, Error = Error> + Send>,
) -> Box<
    Future<
        Item = (Item, Arc<Mutex<Option<Context>>>),
        Error = (Error, Arc<Mutex<Option<Context>>>),
    > + Send,
> {
    let context = Arc::new(Mutex::new(Some(context)));
    Box::new(
        future
            .map(enclose! { (context) |result| (result, context) })
            .map_err(enclose! { (context) |err| (err, context) })
    )
}

pub fn passthrough<Context: 'static + Send, ItemA: 'static, ItemB: 'static, Error: 'static>(
    f: impl Fn(ItemA) -> Box<Future<Item = ItemB, Error = Error> + Send>,
) -> impl Fn(
    (ItemA, Arc<Mutex<Option<Context>>>)
) -> Box<
    Future<
        Item = (ItemB, Arc<Mutex<Option<Context>>>),
        Error = (Error, Arc<Mutex<Option<Context>>>),
    > + Send,
> {
    move |(item, context)| {
        Box::new(
            f(item)
                .map(enclose! { (context) |result| {
                    (result, context)
                }})
                .map_err(enclose! { (context) |err| {
                    (err, context)
                }}),
        )
    }
}

pub fn map<Context: 'static + Send, ItemA, ItemB: 'static, Error: 'static>(
    f: impl Fn(ItemA, Context) -> Box<Future<Item = (ItemB, Context), Error = Error> + Send>,
) -> impl Fn(
    (ItemA, Arc<Mutex<Option<Context>>>)
) -> Box<
    Future<
        Item = (ItemB, Arc<Mutex<Option<Context>>>),
        Error = (Error, Arc<Mutex<Option<Context>>>),
    > + Send,
> {
    move |(item, context_ref)| {
        let context = context_ref.lock().unwrap().take().unwrap();
        Box::new(
            f(item, context)
                .map(enclose! { (context_ref) move |(item, context)| {
                    let mut context_guard = context_ref.lock().unwrap();
                    std::mem::replace(&mut *context_guard, Some(context));
                    (item, context_ref.clone())
                }})
                .map_err(enclose! { (context_ref) |err| {
                    (err, context_ref)
                }}),
        )
    }
}

pub mod dangerous {
    use super::*;

    pub fn finish<Context, T, U>(
        f: impl Fn(T, Context) -> U,
    ) -> impl Fn(
        (T, Arc<Mutex<Option<Context>>>)
    ) -> U {
        move |(item, context_ref)| {
            let context = Arc::try_unwrap(context_ref)
                .ok()
                .unwrap()
                .into_inner()
                .unwrap()
                .unwrap();
            f(item, context)
        }
    }
}
