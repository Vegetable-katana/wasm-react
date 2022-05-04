use js_sys::Function;
use std::{fmt::Debug, ops::Deref, rc::Rc};
use wasm_bindgen::{
  convert::{FromWasmAbi, IntoWasmAbi},
  prelude::Closure,
  JsCast, JsValue,
};

#[derive(Clone)]
pub struct Callback<T, U = ()>(Rc<dyn Fn(T) -> U>);

impl<T, U> Callback<T, U> {
  pub fn new<F: Fn(T) -> U + 'static>(f: F) -> Self {
    Self(Rc::new(f))
  }
}

impl<T, U> Debug for Callback<T, U> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Callback(|_| { ... })")
  }
}

impl<T, U> Deref for Callback<T, U> {
  type Target = Rc<dyn Fn(T) -> U>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T, U> From<Callback<T, U>> for JsValue
where
  T: FromWasmAbi + 'static,
  U: IntoWasmAbi + 'static,
{
  fn from(value: Callback<T, U>) -> Self {
    Closure::wrap(Box::new(move |arg| value.0(arg)) as Box<dyn Fn(T) -> U>)
      .into_js_value()
  }
}

impl<T, U> From<Callback<T, U>> for Function
where
  T: FromWasmAbi + 'static,
  U: IntoWasmAbi + 'static,
{
  fn from(value: Callback<T, U>) -> Self {
    JsValue::from(value).dyn_into().unwrap()
  }
}

pub trait Callable<T, U> {
  fn call(&self, input: T) -> U;
}

impl<T, U, F: Fn(T) -> U> Callable<T, U> for F {
  fn call(&self, input: T) -> U {
    self(input)
  }
}

impl<T, U> Callable<T, U> for Callback<T, U> {
  fn call(&self, input: T) -> U {
    self.0(input)
  }
}

impl Callable<&JsValue, Result<JsValue, JsValue>> for Function {
  fn call(&self, input: &JsValue) -> Result<JsValue, JsValue> {
    self.call1(&JsValue::undefined(), input)
  }
}

impl<T, U: Default, F: Callable<T, U>> Callable<T, U> for Option<F> {
  fn call(&self, input: T) -> U {
    self.as_ref().map(|f| f.call(input)).unwrap_or_default()
  }
}
