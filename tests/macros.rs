//! Tests for macro context bindings.

use librpm::MacroContext;

mod common;

fn ctx() -> MacroContext {
    common::configure();
    MacroContext::default()
}

#[test]
fn test_expand() {
    let result = ctx().expand("%{_arch}").unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_expand_literal() {
    let result = ctx().expand("hello").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_is_defined() {
    let ctx = ctx();
    assert!(ctx.is_defined("_arch"));
    assert!(!ctx.is_defined("nonexistent_macro_xyz_99999"));
}

#[test]
fn test_define_then_expand() {
    let ctx = ctx();
    ctx.define("librpm_test_macro hello_world", 0).unwrap();
    let result = ctx.expand("%{librpm_test_macro}").unwrap();
    assert_eq!(result, "hello_world");
    ctx.pop("librpm_test_macro").unwrap();
}

#[test]
fn test_define_then_is_defined() {
    let ctx = ctx();
    ctx.define("librpm_test_def_check 1", 0).unwrap();
    assert!(ctx.is_defined("librpm_test_def_check"));
    ctx.pop("librpm_test_def_check").unwrap();
    assert!(!ctx.is_defined("librpm_test_def_check"));
}

#[test]
fn test_expand_numeric() {
    let ctx = ctx();
    ctx.define("librpm_test_num 42", 0).unwrap();
    let val = librpm::macro_context::expand_numeric("%{librpm_test_num}");
    assert_eq!(val, 42);
    ctx.pop("librpm_test_num").unwrap();
}
