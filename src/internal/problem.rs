use std::{sync::{atomic::AtomicPtr}, ffi::CStr};
use num_derive::FromPrimitive;

pub(crate) struct Problem(AtomicPtr<librpm_sys::rpmProblem_s>);

impl Problem {
  pub(crate) fn from_ptr(problem: librpm_sys::rpmProblem) -> Self {
    Problem(AtomicPtr::new(problem))
  }
  
  pub(crate) fn data_string(&mut self) -> String {
    let chr = unsafe { librpm_sys::rpmProblemGetStr(*self.0.get_mut()) };
    let cstr = unsafe { CStr::from_ptr(chr) };

    let str = cstr.to_string_lossy().into_owned();
    str
  }

  pub(crate) fn string(&mut self) -> String {
    let chr = unsafe { librpm_sys::rpmProblemString(*self.0.get_mut()) };
    let cstr = unsafe { CStr::from_ptr(chr) };

    let str = cstr.to_string_lossy().into_owned();
    str
  }

  pub(crate) fn problem_type(&mut self) -> ProblemType {
    let num = unsafe { librpm_sys::rpmProblemGetType(*self.0.get_mut()) };

    num::FromPrimitive::from_u32(num).unwrap()
  }

  pub(crate) fn nevr(&mut self) -> String {
    let chr = unsafe { librpm_sys::rpmProblemGetPkgNEVR(*self.0.get_mut()) };
    let cstr = unsafe { CStr::from_ptr(chr) };

    let str = cstr.to_string_lossy().into_owned();
    str
  }

  pub(crate) fn alt_nevr(&mut self) -> String {
    let chr = unsafe { librpm_sys::rpmProblemGetAltNEVR(*self.0.get_mut()) };
    let cstr = unsafe { CStr::from_ptr(chr) };

    let str = cstr.to_string_lossy().into_owned();
    str
  }

  pub(crate) fn equal(one: &mut Problem, two: &mut Problem) -> bool {
    let rc = unsafe { librpm_sys::rpmProblemCompare(*one.0.get_mut(), *two.0.get_mut()) };

    rc == 0
  }

  pub(crate) fn disk_reqirement(&mut self) -> u64 {
    let problem_type = self.problem_type();
    assert_eq!(problem_type, ProblemType::RPMPROB_DISKSPACE);
    assert_eq!(problem_type, ProblemType::RPMPROB_DISKNODES);
    
    unsafe { librpm_sys::rpmProblemGetDiskNeed(*self.0.get_mut()) }
  }
}

// TODO: all of these methods borrow as mut, which means we can't impl traits like PartialEq or Display
// this seems to be related to AtomicPtr not being able to be cloned/copied :/
// we could use a mutex but that leaves two questions
// 1. does librpm already lock the data?
// 2. if not, would using a mutex be too much of a performance impact?
// - L

impl Drop for Problem {
  fn drop(&mut self) {
    unsafe { 
      librpm_sys::rpmProblemFree(*self.0.get_mut());
    }
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub(crate) enum ProblemType {
  /// package ... is for a different architecture
  RPMPROB_BADARCH = librpm_sys::rpmProblemType_e_RPMPROB_BADARCH as isize,
  /// package ... is for a different operating system
  RPMPROB_BADOS = librpm_sys::rpmProblemType_e_RPMPROB_BADOS as isize,
  /// package ... is already installed
  RPMPROB_PKG_INSTALLED = librpm_sys::rpmProblemType_e_RPMPROB_PKG_INSTALLED as isize,
  /// path ... is not relocatable for package ...
  RPMPROB_BADRELOCATE = librpm_sys::rpmProblemType_e_RPMPROB_BADRELOCATE as isize,
  /// package ... has unsatisfied Requires: ...
  RPMPROB_REQUIRES = librpm_sys::rpmProblemType_e_RPMPROB_REQUIRES as isize,
  /// package ... has unsatisfied Conflicts: ...
  RPMPROB_CONFLICT = librpm_sys::rpmProblemType_e_RPMPROB_CONFLICT as isize,
  /// file ... conflicts between attempted installs of ...
  RPMPROB_NEW_FILE_CONFLICT = librpm_sys::rpmProblemType_e_RPMPROB_NEW_FILE_CONFLICT as isize,
  /// file ... from install of ... conflicts with file from package ...
  RPMPROB_FILE_CONFLICT = librpm_sys::rpmProblemType_e_RPMPROB_FILE_CONFLICT as isize,
  /// package ... (which is newer than ...) is already installed
  RPMPROB_OLDPACKAGE = librpm_sys::rpmProblemType_e_RPMPROB_OLDPACKAGE as isize,
  /// installing package ... needs ... on the ... filesystem
  RPMPROB_DISKSPACE = librpm_sys::rpmProblemType_e_RPMPROB_DISKSPACE as isize,
  /// installing package ... needs ... on the ... filesystem
  RPMPROB_DISKNODES = librpm_sys::rpmProblemType_e_RPMPROB_DISKNODES as isize,
  /// package ... is obsoleted by ...
  RPMPROB_OBSOLETES = librpm_sys::rpmProblemType_e_RPMPROB_OBSOLETES as isize,
  /// package did not pass verification
  RPMPROB_VERIFY = librpm_sys::rpmProblemType_e_RPMPROB_VERIFY as isize,
}