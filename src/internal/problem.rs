use std::sync::atomic::AtomicPtr;
use num_derive::FromPrimitive;

pub(crate) struct Problem(AtomicPtr<librpm_sys::rpmProblem_s>);

impl Problem {
  pub(crate) fn create(type_: ProblemType, pkgNEVR: &str, ) -> Self {
    // librpm_sys::rpmProblemCreate(type_, pkgNEVR, key, altNEVR, str_, number)
  }
}

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