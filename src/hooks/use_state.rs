use super::{use_ref, UseRef};
use crate::{callback::Void, react_bindings, Callable};
use js_sys::Function;
use std::{
  fmt::Debug,
  ops::{Deref, DerefMut},
};
use wasm_bindgen::UnwrapThrowExt;

pub struct UseState<T>(UseRef<Option<T>>, Function);

impl<T: 'static> UseState<T> {
  pub fn update(&mut self, mutator: impl FnOnce(&mut T)) {
    mutator(self.0.deref_mut().current.as_mut().unwrap_throw());

    self.1.call(&Void.into());
  }
}

impl<T> Clone for UseState<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone(), self.1.clone())
  }
}

impl<T> Deref for UseState<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.0.deref().current.as_ref().unwrap_throw()
  }
}

impl<T: Debug> Debug for UseState<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.deref().fmt(f)
  }
}

pub fn use_state<T: 'static>(init: impl FnOnce() -> T) -> UseState<T> {
  let mut inner_ref = use_ref(None);

  if inner_ref.current.is_none() {
    inner_ref.current = Some(init());
  }

  let update = react_bindings::use_rust_state();

  UseState(inner_ref, update)
}
