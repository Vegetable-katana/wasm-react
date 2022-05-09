use crate::{
  callback::{Callback, Void},
  react_bindings, Persisted, PersistedOrigin,
};
use js_sys::Reflect;
use std::{fmt::Debug, marker::PhantomData};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};

/// Allows access to the underlying data persisted with [`use_ref()`].
pub struct RefContainer<T>(*mut T);

impl<T> RefContainer<T> {
  /// Returns a reference to the underlying data.
  pub fn current(&self) -> &T {
    Box::leak(unsafe { Box::from_raw(self.0) })
  }

  /// Returns a mutable reference to the underlying data.
  pub fn current_mut(&mut self) -> &mut T {
    Box::leak(unsafe { Box::from_raw(self.0) })
  }

  /// Sets the underlying data to the given value.
  pub fn set_current(&mut self, value: T) {
    *self.current_mut() = value;
  }
}

impl<T> Persisted for RefContainer<T> {
  fn ptr(&self) -> PersistedOrigin {
    PersistedOrigin
  }
}

impl<T: Debug> Debug for RefContainer<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("RefContainer")
      .field("current", self.current())
      .finish()
  }
}

impl<T> Clone for RefContainer<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

pub(crate) fn use_ref_with_unmount_handler<T: 'static>(
  init: T,
  unmount_handler: impl FnOnce(&mut RefContainer<T>) + 'static,
) -> RefContainer<T> {
  let ptr = react_bindings::use_rust_ref(
    Callback::once(move |_: Void| Box::into_raw(Box::new(init))).as_ref(),
  ) as *mut T;

  // This callback will always be called exactly one time.
  react_bindings::use_unmount_handler(&Closure::once_into_js(
    move |unmounted: bool| {
      if unmounted {
        unmount_handler(&mut RefContainer(ptr));
        drop(unsafe { Box::from_raw(ptr) });
      }
    },
  ));

  RefContainer(ptr)
}

/// This is the main hook to persist Rust data through the entire lifetime of
/// the component.
///
/// Whenever the component is unmounted by React, the data will also be dropped.
/// Keep in mind that [`use_ref()`] can only be mutated in Rust. If you need a
/// ref to hold a DOM element, use [`use_js_ref()`] instead.
///
/// The component will not rerender when you mutate the underlying data. If you
/// want that, use [`use_state()`](crate::hooks::use_state()) instead.
///
/// # Example
///
/// ```
/// # use wasm_react::{*, hooks::*};
/// # struct MyData { value: &'static str };
/// # struct MyComponent { value: &'static str };
/// impl Component for MyComponent {
///   /* ... */
///   # fn name() -> &'static str { "" }
///
///   fn render(&self) -> VNode {
///     let mut ref_container = use_ref(MyData {
///       value: "Hello World!"
///     });
///
///     use_effect(|| {
///       ref_container.current_mut().value = self.value;
///
///       || ()
///     }, Deps::some(self.value));
///
///     h!(div).build(children![
///       ref_container.current().value
///     ])
///   }
/// }
/// ```
pub fn use_ref<T: 'static>(init: T) -> RefContainer<T> {
  use_ref_with_unmount_handler(init, |_| ())
}

/// Allows access to the underlying JS data persisted with [`use_js_ref()`].
pub struct JsRefContainer<T>(JsValue, PhantomData<T>);

impl<T: JsCast> JsRefContainer<T> {
  /// Returns the underlying typed JS data.
  pub fn current(&self) -> Option<T> {
    self.current_untyped().dyn_into::<T>().ok()
  }

  /// Returns the underlying JS data as [`JsValue`].
  pub fn current_untyped(&self) -> JsValue {
    Reflect::get(&self.0, &"current".into()).unwrap_throw()
  }

  /// Sets the underlying JS data.
  pub fn set_current(&self, value: Option<&T>) {
    Reflect::set(
      &self.0,
      &"current".into(),
      value.map(|t| t.as_ref()).unwrap_or(&JsValue::null()),
    )
    .unwrap_throw();
  }
}

impl<T> Persisted for JsRefContainer<T> {
  fn ptr(&self) -> PersistedOrigin {
    PersistedOrigin
  }
}

impl<T> Debug for JsRefContainer<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("JsRefContainer").field(&self.0).finish()
  }
}

impl<T> Clone for JsRefContainer<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone(), PhantomData)
  }
}

impl<T> AsRef<JsValue> for JsRefContainer<T> {
  fn as_ref(&self) -> &JsValue {
    &self.0
  }
}

impl<T> From<JsRefContainer<T>> for JsValue {
  fn from(value: JsRefContainer<T>) -> Self {
    value.0
  }
}

/// A binding to `React.useRef()`. This hook can persist JS data through the
/// entire lifetime of the component.
///
/// Use this if you need JS to set the ref value. If you only need to mutate the
/// data from Rust, use [`use_ref()`] instead.
///
/// # Example
///
/// ```
/// # use wasm_react::{*, hooks::*};
/// # struct MyComponent;
/// impl Component for MyComponent {
///   /* ... */
///   # fn name() -> &'static str { "" }
///
///   fn render(&self) -> VNode {
///     let input_element = use_js_ref(None);
///
///     h!(div)
///       .build(children![
///         h!(input)
///           .ref_container(&input_element)
///           .html_type("text")
///           .build(children![])
///       ])
///   }
/// }
/// ```
pub fn use_js_ref<T: JsCast>(init: Option<T>) -> JsRefContainer<T> {
  let ref_container = react_bindings::use_ref(
    &init.map(|init| init.into()).unwrap_or(JsValue::null()),
  );

  JsRefContainer(ref_container, PhantomData)
}
