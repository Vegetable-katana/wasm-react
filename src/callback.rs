//! This module provides structs to pass Rust closures to JS.

use js_sys::Function;
use std::{fmt::Debug, rc::Rc};
use wasm_bindgen::{
  convert::{FromWasmAbi, IntoWasmAbi},
  describe::WasmDescribe,
  prelude::Closure,
  JsCast, JsValue, UnwrapThrowExt,
};

/// A helper struct to simulate a [`Callback`] with no input arguments.
///
/// ```
/// let callback: Callback<Void> = Callback::new(|_: Void| ());
/// ```
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Void;

impl WasmDescribe for Void {
  fn describe() {
    JsValue::describe()
  }
}

impl FromWasmAbi for Void {
  type Abi = <JsValue as FromWasmAbi>::Abi;

  unsafe fn from_abi(js: Self::Abi) -> Self {
    JsValue::from_abi(js);
    Void
  }
}

impl From<Void> for JsValue {
  fn from(_: Void) -> Self {
    JsValue::undefined()
  }
}

/// This is a simplified wrapper around a [`Closure`] which represents a Rust
/// closure that can be called from JS.
///
/// It only supports closures with one and only one input argument and some
/// return value. Memory management is handled by Rust. Whenever Rust drops
/// the [`Callback`], the function cannot be called from JS anymore.
///
/// This can be used in conjunction with the [`use_callback`](crate::hooks::use_callback())
/// hook to make the callback valid for the entire lifetime of a component.
pub struct Callback<T, U = ()>(Rc<Closure<dyn FnMut(T) -> U>>);

impl<T, U> Callback<T, U> {
  /// Constructs a new [`Callback`] from a [`FnMut`].
  pub fn new(mut f: impl FnMut(T) -> U + 'static) -> Self
  where
    T: FromWasmAbi + 'static,
    U: IntoWasmAbi + 'static,
  {
    Self(Rc::new(Closure::wrap(
      Box::new(move |arg| f(arg)) as Box<dyn FnMut(T) -> U>
    )))
  }

  /// Constructs a new [`Callback`] from a [`FnOnce`].
  pub fn once(f: impl FnOnce(T) -> U + 'static) -> Self
  where
    T: FromWasmAbi + 'static,
    U: IntoWasmAbi + 'static,
  {
    Self(Rc::new(Closure::once(move |arg| f(arg))))
  }
}

impl<T> Default for Callback<T, ()>
where
  T: FromWasmAbi + 'static,
{
  fn default() -> Self {
    Self::new(|_| ())
  }
}

impl<T, U> Debug for Callback<T, U> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Callback(|_| { ... })")
  }
}

impl<T, U> PartialEq for Callback<T, U> {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.0, &other.0)
  }
}

impl<T, U> Eq for Callback<T, U> {}

impl<T, U> Clone for Callback<T, U> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<T, U> AsRef<JsValue> for Callback<T, U> {
  fn as_ref(&self) -> &JsValue {
    (*self.0).as_ref()
  }
}

impl<T, U> AsRef<Function> for Callback<T, U> {
  fn as_ref(&self) -> &Function {
    (*self.0).as_ref().dyn_ref::<Function>().unwrap_throw()
  }
}

/// A trait for callable structs with one and only one input argument and some
/// return value.
pub trait Callable<T, U = ()> {
  /// Calls the struct with the given input argument.
  fn call(&self, arg: T) -> U;
}

impl<T, U, F: Fn(T) -> U> Callable<T, U> for F {
  fn call(&self, arg: T) -> U {
    self(arg)
  }
}

impl<T> Callable<T, ()> for Callback<T, ()>
where
  T: Into<JsValue>,
{
  fn call(&self, arg: T) {
    (self.as_ref() as &Function)
      .call1(&JsValue::undefined(), &arg.into())
      .unwrap_throw();
  }
}

impl<T> Callable<T, JsValue> for Callback<T, JsValue>
where
  T: Into<JsValue>,
{
  fn call(&self, arg: T) -> JsValue {
    (self.as_ref() as &Function)
      .call1(&JsValue::undefined(), &arg.into())
      .unwrap_throw()
  }
}

impl Callable<&JsValue, JsValue> for Function {
  fn call(&self, arg: &JsValue) -> JsValue {
    self.call1(&JsValue::undefined(), arg).unwrap_throw()
  }
}

impl<T, U: Default, F: Callable<T, U>> Callable<T, U> for Option<F> {
  fn call(&self, arg: T) -> U {
    self.as_ref().map(|f| f.call(arg)).unwrap_or_default()
  }
}
