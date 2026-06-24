Name:           test-package
Version:        1.0
Release:        1%{?dist}
Summary:        A test package for librpm.rs

License:        MIT
Source0:        test-package-1.0.tar.gz
Patch0:         fix-build.patch

%description
A test package used by the librpm.rs test suite.

%prep
echo prep

%build
echo build

%install
echo install

%files
