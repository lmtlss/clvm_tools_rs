use std::os::raw::{c_char};
use std::ffi::{CString, CStr};
use std::rc::Rc;
use crate::classic::clvm::__type_compatibility__::{Bytes, BytesFromType, Stream, sha256};
use crate::classic::clvm_tools::binutils::{assemble, disassemble};
use crate::classic::clvm_tools::ir::reader::read_ir;
use crate::classic::clvm_tools::sha256tree::sha256tree;
use clvm_rs::allocator::{Allocator, NodePtr};
use clvm_rs::chia_dialect::ChiaDialect;
use clvm_rs::reduction::EvalErr;
use clvm_rs::run_program::run_program;
use crate::classic::clvm::serialize::{sexp_from_stream, sexp_to_stream, SimpleCreateCLVMObject};
use crate::classic::clvm::sexp::enlist;
use crate::classic::clvm_tools::binutils::assemble_from_ir;
use crate::classic::clvm::casts::{bigint_to_bytes, TConvertOption};
use num_bigint::ToBigInt;
use crate::classic::clvm::sexp::rest;
use crate::classic::clvm::sexp::first;

#[no_mangle]
fn get_string(string: *const c_char) -> String {
  let c_str = unsafe { CStr::from_ptr(string) };
  let recipient = match c_str.to_str() {
      Err(_) => "there",
      Ok(string) => string,
  };
  recipient.to_owned()
}

fn get_program(allocator: &mut Allocator, string: *const c_char) -> NodePtr {
  let str = get_string(string);
  let mut stream = Stream::new(Some(Bytes::new(Some(BytesFromType::Hex(
    str,
  )))));
  let sexp = sexp_from_stream(allocator, &mut stream, Box::new(SimpleCreateCLVMObject {})).unwrap();
  let prog_bin = sexp_as_bin(allocator,  sexp.1);
  sexp.1
}


#[no_mangle]
pub extern fn treehash(to: *const c_char) -> *mut c_char {
  let mut allocator = Allocator::new();
  let prog = get_program(&mut allocator, to);
  let treehash = sha256tree(&mut allocator, prog);
  CString::new(treehash.hex()).unwrap().into_raw()
}

#[no_mangle]
pub extern fn int_to_bytes(value: i64) -> *mut c_char {
  let big = value.to_bigint().unwrap();
  let intbytes = bigint_to_bytes(&big, Some(TConvertOption { signed: true })).unwrap();
  CString::new(intbytes.hex()).unwrap().into_raw()
}

pub fn sexp_as_bin(allocator: &mut Allocator, sexp: NodePtr) -> Bytes {
  let mut f = Stream::new(None);
  sexp_to_stream(allocator, sexp, &mut f);
  f.get_value()
}

#[no_mangle]
pub extern fn c_curry(program: *const c_char, args: *const c_char) -> *mut c_char {
  let mut allocator = Allocator::new();
  let prog = get_program(&mut allocator, program);
  let prog_bin = sexp_as_bin(&mut allocator, prog);

  let binding = get_string(args);
  let args = binding.as_bytes();

  let ir_src = read_ir(&binding).map_err(|s| EvalErr(allocator.null(), s)).unwrap();
  let assembled_sexp = assemble_from_ir(&mut allocator, Rc::new(ir_src)).unwrap();
  let args = assembled_sexp;

  let curried = curry(&mut allocator, prog, args);
  let prog_bin = sexp_as_bin(&mut allocator, curried);

  CString::new(prog_bin.hex()).unwrap().into_raw()

}

fn curry(allocator: &mut Allocator, program: NodePtr, args: NodePtr) -> NodePtr {
  let CURRY_OBJ_CODE = assemble(
  allocator, "(a (q #a 4 (c 2 (c 5 (c 7 0)))) (c (q (c (q . 2) (c (c (q . 1) 5) (c (a 6 (c 2 (c 11 (q 1)))) 0))) #a (i 5 (q 4 (q . 4) (c (c (q . 1) 9) (c (a 6 (c 2 (c 13 (c 11 0)))) 0))) (q . 11)) 1) 1))"
  ).unwrap();
  let dialect = ChiaDialect::new(0);
  let sexp = allocator.sexp(args);

  let input_args = enlist(allocator, &[args]).unwrap();
  let dis1 = disassemble(allocator, input_args);

  let input_sexp = enlist(allocator, &[program, args]).unwrap();
  let dis = disassemble(allocator, input_sexp);
  let run_result = run_program(allocator, &dialect, CURRY_OBJ_CODE, input_sexp, 18446744073709551615, None).unwrap();
  run_result.1
}

#[no_mangle]
pub extern fn swift_assemble(program: *const c_char) -> *mut c_char {
  let mut allocator = Allocator::new();
  let clvm = get_string(program);

  let assembled = assemble(
    &mut allocator, &clvm
    ).unwrap();
  let prog_bin = sexp_as_bin(&mut allocator, assembled);

  CString::new(prog_bin.hex()).unwrap().into_raw()
}

#[no_mangle]
pub extern fn swift_disassemble(program: *const c_char) -> *mut c_char {
  let mut allocator = Allocator::new();
  let clvm: NodePtr = get_program(&mut allocator, program);

  let dis = disassemble(&mut allocator, clvm);
  CString::new(dis).unwrap().into_raw()
}

#[no_mangle]
pub extern fn swift_rest(program: *const c_char) -> *mut c_char {
  let mut allocator = Allocator::new();
  let program = get_program(&mut allocator, program);
  let rest_result = rest(& mut allocator, program);
  let rest = rest_result.unwrap();
  let prog_bin = sexp_as_bin(&mut allocator, rest);
  CString::new(prog_bin.hex()).unwrap().into_raw()
}

#[no_mangle]
pub extern fn swift_first(program: *const c_char) -> *mut c_char {
  let mut allocator = Allocator::new();
  let program = get_program(&mut allocator, program);
  let first_result = first(& mut allocator, program);
  let first = first_result.unwrap();
  let prog_bin = sexp_as_bin(&mut allocator, first);
  CString::new(prog_bin.hex()).unwrap().into_raw()
}

#[no_mangle]
pub extern fn swift_run(program_str: *const c_char, solution_str: *const c_char,) -> *mut c_char {
  let mut allocator = Allocator::new();
  let program: NodePtr = get_program(&mut allocator, program_str);
  let solution: NodePtr = get_program(&mut allocator, solution_str);
  let dialect = ChiaDialect::new(0);
  let run_result = run_program(&mut allocator, &dialect, program, solution, 18446744073709551615, None).unwrap();
  let dis = disassemble(&mut allocator, run_result.1);
  CString::new(dis).unwrap().into_raw()
}
