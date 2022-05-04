use js_sys::{Array, Function};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/react-bindings.js")]
extern "C" {
  #[wasm_bindgen(js_namespace = React, js_name = createElement)]
  pub fn create_element(
    name: JsValue,
    props: JsValue,
    children: Array,
  ) -> JsValue;

  #[wasm_bindgen(js_name = useRustState)]
  pub fn use_rust_state(create: &dyn Fn() -> usize, on_free: Function) -> Array;

  #[wasm_bindgen(js_namespace = React, js_name = useEffect)]
  pub fn use_effect(f: Function, deps: JsValue);

  #[wasm_bindgen(js_name = createComponent)]
  pub(crate) fn create_component(name: &str, props: JsValue) -> JsValue;

  #[wasm_bindgen(js_name = cast)]
  pub(crate) fn cast_into_usize(value: JsValue) -> usize;
}
