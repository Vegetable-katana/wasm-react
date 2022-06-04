/// A convenience macro to [`create_element()`](crate::create_element()) for
/// creating HTML element nodes.
///
/// Returns an [`H`](crate::props::H) struct that provides auto-completion for
/// HTML attributes and events.
///
/// # Example
///
/// ```
/// # use wasm_react::*;
/// # fn f() -> VNode {
/// h!(div)
///   .attr("id", &"app".into())
///   .build(c![
///     h!(h1).build(c!["Hello World!"])
///   ])
/// # }
///
/// // <div id="app"><h1>Hello World!</h1></div>
///
/// # fn g() -> VNode {
/// h!("web-component")
///   .build(c!["Hello World!"])
/// # }
///
/// // <web-component>Hello World!</web-component>
/// ```
///
/// It is also possible to add an id and/or classes to the element using a terse
/// notation. You can use the same syntax as [`classnames!`](crate::classnames!).
///
/// ```
/// # use wasm_react::*;
/// # fn f() -> VNode {
/// h!(div[#"app"."some-class"."warning"])
///   .build(c!["This is a warning!"])
/// # }
///
/// // <div id="app" class="some-class warning">This is a warning!</div>
/// ```
#[macro_export]
macro_rules! h {
  ($tag:literal $( [$( #$id:literal )? $( .$( $classnames:tt )+ )?] )?) => {
    $crate::props::H::new($tag) $(
      $( .id($id) )?
      $( .class_name(&$crate::classnames![.$( $classnames )+]) )?
    )?
  };
  ($tag:ident $( [$( #$id:literal )? $( .$( $classnames:tt )+ )?] )?) => {
    $crate::props::H::new(stringify!($tag)) $(
      $( .id($id) )?
      $( .class_name(&$crate::classnames![.$( $classnames )+]) )?
    )?
  };
}

/// This macro will take various objects of [`Into<VNode>`](crate::VNode) or
/// [`Iterator<Item = VNode>`](crate::VNode) and builds a [`VNodeList`].
///
/// [`VNodeList`]: crate::VNodeList
///
/// # Example
///
/// ```
/// # use wasm_react::*;
/// #
/// # struct SomeComponent { some_prop: () }
/// # impl Component for SomeComponent {
/// #   fn render(&self) -> VNode { VNode::empty() }
/// # }
/// #
/// # fn f(some_prop: (), vec: Vec<&str>, some_bool: bool) -> VNode {
/// h!(div).build(c![
///   "Counter: ", 5,
///
///   SomeComponent {
///     some_prop,
///   }
///   .build(),
///
///   some_bool.then(||
///     h!(p).build(c!["Conditional rendering"]),
///   ),
///
///   h!(h1).build(c!["Hello World"]),
///
///   ..vec.iter()
///     .map(|x| h!(p).build(c![*x])),
/// ])
/// # }
/// ```
#[macro_export]
macro_rules! c {
  [@single $list:ident <<] => {};

  // Handle iterators
  [@single $list:ident << ..$vnode_list:expr $(, $( $tail:tt )* )?] => {
    $list.extend($vnode_list);
    $crate::c![@single $list << $( $( $tail )* )?];
  };

  // Handle `Into<VNode>`
  [@single $list:ident << $into_vnode:expr $(, $( $tail:tt )* )?] => {
    $list.push(&$into_vnode.into());
    $crate::c![@single $list << $( $( $tail )* )?];
  };

  [] => {
    $crate::VNodeList::new()
  };
  [$( $tt:tt )*] => {
    {
      let mut list = $crate::VNodeList::new();
      $crate::c![@single list << $( $tt )*];
      list
    }
  };
}

/// Constructs a [`String`] based on various types that implement
/// [`Classnames`](crate::props::Classnames).
///
/// # Example
///
/// ```
/// # use wasm_react::*;
/// assert_eq!(
///   classnames![."button"."blue"],
///   "button blue ".to_string(),
/// );
///
/// let blue = false;
/// let disabled = true;
///
/// assert_eq!(
///   classnames![."button".blue.disabled],
///   "button disabled ".to_string(),
/// );
///
/// let is_blue = Some("blue");
/// let disabled = "disabled".to_string();
///
/// assert_eq!(
///   classnames![."button".{is_blue}.{&disabled}],
///   "button blue disabled ",
/// );
/// ```
#[macro_export]
macro_rules! classnames {
  [@single $result:ident <<] => {};

  // Handle string literals
  [@single $result:ident << .$str:literal $( $tail:tt )*] => {
    $crate::props::Classnames::append_to(&$str, &mut $result);
    $crate::classnames![@single $result << $( $tail ) *];
  };

  // Handle boolean variables
  [@single $result:ident << .$bool:ident $( $tail:tt )*] => {
    $crate::props::Classnames::append_to(
      &$bool.then(|| stringify!($bool)),
      &mut $result
    );
    $crate::classnames![@single $result << $( $tail ) *];
  };

  // Handle block expressions
  [@single $result:ident << .$block:block $( $tail:tt )*] => {
    $crate::props::Classnames::append_to(&$block, &mut $result);
    $crate::classnames![@single $result << $( $tail ) *];
  };

  [] => {
    String::new()
  };
  [$( $tt:tt )*] => {
    {
      let mut result = String::new();
      $crate::classnames![@single result << $( $tt )*];
      result
    }
  };
}

/// This macro can be used to expose your [`Component`](crate::Component) for JS
/// consumption via `wasm-bindgen`.
///
/// Requirement is that you implement the [`TryFrom<JsValue, Error = JsValue>`](core::convert::TryFrom)
/// trait on your component and that you do not export anything else that has
/// the same name as your component.
///
/// Therefore, it is only recommended to use this macro if you're writing a
/// library for JS consumption only, or if you're writing a standalone
/// application, since this will pollute the export namespace, which isn't
/// desirable if you're writing a library for Rust consumption only.
///
/// # Example
///
/// Implement [`TryFrom<JsValue, Error = JsValue>`](core::convert::TryFrom) on
/// your component and export it:
///
/// ```
/// # use wasm_react::*;
/// # use wasm_bindgen::prelude::*;
/// # use js_sys::Reflect;
/// #
/// pub struct Counter {
///   counter: i32,
/// }
///
/// impl Component for Counter {
///   # fn render(&self) -> VNode { VNode::empty() }
///   /* ... */
/// }
///
/// impl TryFrom<JsValue> for Counter {
///   type Error = JsValue;
///
///   fn try_from(value: JsValue) -> Result<Self, Self::Error> {
///     let diff = Reflect::get(&value, &"counter".into())?
///       .as_f64()
///       .ok_or(JsError::new("`counter` property not found"))?;
///
///     Ok(Counter { counter: diff as i32 })
///   }
/// }
///
/// export_components! { Counter }
/// ```
///
/// In JS, you can use it like any other component:
///
/// ```js
/// import React from "react";
/// import init, { Counter } from "./path/to/pkg/project.js";
///
/// function SomeOtherJsComponent(props) {
///   return (
///     <div>
///       <Counter counter={0} />
///     </div>
///   );
/// }
/// ```
#[macro_export]
macro_rules! export_components {
  {} => {};
  { $component:ident $( , $( $tail:tt )* )? } => {
    $crate::export_components! { $component as $component $( , $( $tail )* )? }
  };
  { $component:ty as $name:ident $( , $( $tail:tt )* )? } => {
    $crate::paste! {
      #[allow(non_snake_case)]
      #[allow(dead_code)]
      #[doc(hidden)]
      #[wasm_bindgen::prelude::wasm_bindgen(js_name = $name)]
      pub fn [<__WasmReact_Export_ $name>](
        props: wasm_bindgen::JsValue,
      ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>
      where
        $component: $crate::Component
          + TryFrom<wasm_bindgen::JsValue, Error = wasm_bindgen::JsValue>,
      {
        let component = $component::try_from(props)?;
        Ok($crate::Component::render(&component).into())
      }
    }

    $( $crate::export_components! { $( $tail )* } )?
  };
}

#[macro_export]
macro_rules! import_components {
  { #[$meta:meta] } => {};
  {
    #[$meta:meta]
    $vis:vis $component:ident $( , $( $tail:tt )* )?
  } => {
    $crate::import_components! {
      #[$meta]
      $component as $vis $component $( , $( $tail:tt )* )?
    }
  };
  {
    #[$meta:meta]
    $component:ident as $vis:vis $name:ident $( , $( $tail:tt )* )?
  } => {
    $crate::paste! {
      #[$meta]
      extern "C" {
        #[wasm_bindgen(js_name = $component)]
        static [<__WASMREACT_IMPORT_ $name:upper>]: JsValue;
      }

      #[allow(non_snake_case)]
      $vis fn $name(
        props: &$crate::props::Props
      ) -> $crate::JsComponentWrapper<'_> {
        $crate::JsComponentWrapper::new(
          &[<__WASMREACT_IMPORT_ $name:upper>],
          props,
        )
      }
    }

    $( $crate::import_components! { #[$meta] $( $tail )* } )?
  };
}
