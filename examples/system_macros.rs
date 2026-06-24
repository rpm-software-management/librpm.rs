/// Print distro packaging details inherited from the system RPM macros.
///
/// Run with: cargo run --example system_macros

// RPM macro syntax:
//   %{name}          - expand macro (literal passthrough if undefined)
//   %{?name}         - expand macro (empty string if undefined)
//   %{?name:text}    - expand to "text" if name IS defined
//   %{!?name:text}   - expand to "text" if name is NOT defined
fn main() {
    librpm::init().expect("failed to initialize librpm");
    let ctx = librpm::MacroContext::default();

    println!("=== Platform ===");
    println!("  arch:    {}", ctx.expand("%{_arch}").unwrap());
    println!("  os:      {}", ctx.expand("%{_os}").unwrap());
    // Detect the distro family: whichever macro is defined expands its text,
    // the others silently contribute nothing
    println!(
        "  family:  {}",
        ctx.expand("%{?fedora:Fedora}%{?rhel:RHEL}%{?suse_version:SUSE}%{!?fedora:%{!?rhel:%{!?suse_version:Unknown}}}").unwrap()
    );
    println!("  dist:         {}", ctx.expand("%{?dist}").unwrap());
    println!("  dist name:    {}", ctx.expand("%{?dist_name}").unwrap());
    println!("  dist vendor:  {}", ctx.expand("%{?dist_vendor}").unwrap());
    println!(
        "  dist url:     {}",
        ctx.expand("%{?dist_home_url}").unwrap()
    );
    // Use is_defined() for conditional execution
    if ctx.is_defined("fedora") {
        println!("  Fedora version: {}", ctx.expand("%{fedora}").unwrap());
    } else if ctx.is_defined("rhel") {
        println!("  RHEL version: {}", ctx.expand("%{rhel}").unwrap());
    } else if ctx.is_defined("suse_version") {
        println!("  SUSE version: {}", ctx.expand("%{suse_version}").unwrap());
    }

    println!("\n=== Filesystem Layout ===");
    println!("  prefix:     {}", ctx.expand("%{_prefix}").unwrap());
    println!("  bindir:     {}", ctx.expand("%{_bindir}").unwrap());
    println!("  libdir:     {}", ctx.expand("%{_libdir}").unwrap());
    println!("  sysconfdir: {}", ctx.expand("%{_sysconfdir}").unwrap());
    println!("  datadir:    {}", ctx.expand("%{_datadir}").unwrap());

    println!("\n=== Build Paths ===");
    println!("  topdir:     {}", ctx.expand("%{_topdir}").unwrap());
    println!("  sourcedir:  {}", ctx.expand("%{_sourcedir}").unwrap());
    println!("  builddir:   {}", ctx.expand("%{_builddir}").unwrap());
    println!("  rpmdir:     {}", ctx.expand("%{_rpmdir}").unwrap());
    println!("  srcrpmdir:  {}", ctx.expand("%{_srcrpmdir}").unwrap());
}
