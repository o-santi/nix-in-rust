use std::{collections::{btree_map::Values, HashMap}, ffi::{c_char, c_void, CStr}, ptr::NonNull};

use crate::{bindings::{nix_c_context, nix_copy_value, nix_init_bool, nix_init_int, nix_value_force, nix_version_get, EvalState, Value}, eval::{NixEvalState, RawValue, StateWrapper}, store::{NixContext, NixStore}, term::{NixEvalError, NixTerm}};

pub fn get_nix_version() -> String {
  unsafe {
    let version = nix_version_get();
    CStr::from_ptr(version)
      .to_str()
      .expect("nix version should be valid utf8")
      .to_owned()
  }
}

pub unsafe extern "C" fn callback_get_vec_u8(
    start: *const c_char,
    n: std::os::raw::c_uint,
    user_data: *mut c_void,
) {
    let ret = user_data as *mut Vec<u8>;
    let slice = std::slice::from_raw_parts(start as *const u8, n as usize);
    if !(*ret).is_empty() {
        panic!("callback_get_vec_u8: slice must be empty. Were we called twice?");
    }
    (*ret).extend_from_slice(slice);
}

pub extern "C" fn read_into_hashmap(map: *mut c_void, outname: *const c_char, out: *const c_char) {
  let map: &mut HashMap<String, String> = unsafe { std::mem::transmute(map) };
  let key = unsafe { CStr::from_ptr(outname)}.to_str().expect("nix key should be valid string");
  let path = unsafe { CStr::from_ptr(out)}.to_str().expect("nix path should be valid string");
  map.insert(key.to_string(), path.to_string());
}

pub unsafe extern "C" fn call_rust_closure<F>(
  func: *mut c_void,
  context: *mut nix_c_context,
  state: *mut EvalState,
  args: *mut *mut Value,
  mut ret: *mut Value
)
where F: Fn(NixTerm) -> Result<NixTerm, NixEvalError> {
  let closure: &Box<F> = std::mem::transmute(func);
  let ctx = NixContext::default();
  let store = NixStore::new(ctx, "");
  let state = NonNull::new(state).expect("state should never be null");
  let state = NixEvalState {
    store, _eval_state: std::rc::Rc::new(StateWrapper(state)),
  };
  let value = {
    nix_value_force(state.store.ctx.ptr(), state.state_ptr(), *args);
    NonNull::new(*args).expect("Expected at least one argument")
  };
  state.store.ctx.check_call().unwrap();
  let rawvalue = RawValue::from_raw(value, state.clone());
  let argument: NixTerm = rawvalue.try_into().unwrap();
  let func_ret: NixTerm = closure(argument).expect("Closure returned an error");
  let rawvalue: RawValue = func_ret.to_raw_value(&state);
  // nix_init_bool(state.store.ctx.ptr(), ret, false);
  // ret.write_volatile(*rawvalue.value);
}

pub fn eval_from_str(str: &str) -> anyhow::Result<NixTerm> {
  let context = NixContext::default();
  let store = NixStore::new(context, "");
  let mut state = NixEvalState::new(store);
  state.eval_from_string(str)
}
