use crate::{react_bindings, Context};
use std::{rc::Rc, thread::LocalKey};

/// Allows access to the current context value of the given context.
///
/// See [`create_context()`](crate::create_context()) for usage.
pub fn use_context<T>(context: &'static LocalKey<Context<T>>) -> Rc<T> {
  let mut result = None;

  context.with(|context| {
    react_bindings::use_rust_context(
      context.as_ref(),
      &mut |ref_container_value| {
        result = Some(
          ref_container_value
            .value::<T>()
            .expect("mismatched context type"),
        );
      },
    );
  });

  result.expect("callback was not called")
}
