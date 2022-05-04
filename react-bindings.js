import { __WasmReact_ComponentWrapper } from "../../wasm_react.js";
import * as WasmExports from "../../wasm_react.js";

let components = {};

export let React = undefined;

export function setReact(value) {
  React = value;
}

export function registerComponent(name) {
  if (components[name] == null) {
    // This curious construction is needed to ensure that the components show up
    // with their names correctly in the React Developer Tools
    Object.assign(components, {
      [name]: (props = {}) => {
        if (props.component instanceof __WasmReact_ComponentWrapper) {
          // This is a component implemented using the Component trait, i.e.
          // with a Rust struct as props
          let component = props.component;

          // We need to free up the memory on Rust side whenever the old props
          // are replaced with new ones.
          React.useEffect(() => () => component.free(), [component]);

          return __WasmReact_ComponentWrapper.render(component);
        } else {
          // TODO: This is a component not implemented using the Component trait
          return WasmExports[name].render(props);
        }
      },
    });
  }
}

export function getComponent(name) {
  registerComponent(name);
  return components[name];
}

export function createComponent(name, props) {
  return React.createElement(getComponent(name), props);
}

export function useRustState(create, onFree) {
  // We only maintain a pointer to the state struct
  let [state, setState] = React.useState(() => ({ ptr: create() }));
  // Let Rust free up the memory whenever the component unmounts
  React.useEffect(() => () => onFree(state.ptr), []);

  return [
    state.ptr,
    (mutator) =>
      setState((state) => {
        mutator();
        return { ...state };
      }),
  ];
}

export function cast(x) {
  return x;
}
