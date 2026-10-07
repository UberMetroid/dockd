%{!?_userunitdir: %global _userunitdir %{_prefix}/lib/systemd/user}

Name:           dockd
Version:        0.1.0
Release:        1%{?dist}
Summary:        Pure-standard-library Rust application dock and window manager daemon for Omarchy

License:        GPL-2.0-or-later
URL:            https://github.com/UberMetroid/dockd
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo >= 1.80
BuildRequires:  rust >= 1.80
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc

Requires:       systemd
Requires:       glibc

%description
dockd is a high-performance desktop dock and window manager daemon for Omarchy
and Hyprland. Built strictly in pure-standard-library Rust with zero crates.io
dependencies and native systemd user session integration.

%prep
%autosetup

%build
cargo build --release

%check
cargo test

%install
install -D -p -m 0755 target/release/dockd %{buildroot}%{_bindir}/dockd
install -D -p -m 0644 systemd/user/dockd.service %{buildroot}%{_userunitdir}/dockd.service
install -D -p -m 0644 systemd/user/dockd.socket %{buildroot}%{_userunitdir}/dockd.socket

install -d -m 0755 %{buildroot}%{_datadir}/omarchy/plugins/org.ubermetroid.dockd
cp -rf plugin/* %{buildroot}%{_datadir}/omarchy/plugins/org.ubermetroid.dockd/
install -m 0755 uninstall.sh %{buildroot}%{_datadir}/omarchy/plugins/org.ubermetroid.dockd/uninstall.sh
ln -sf ../share/omarchy/plugins/org.ubermetroid.dockd/uninstall.sh %{buildroot}%{_bindir}/dockd-uninstall


%post
%{?systemd_user_post: %systemd_user_post dockd.socket}

%preun
%{?systemd_user_preun: %systemd_user_preun dockd.socket dockd.service}

%postun
%{?systemd_user_postun_with_restart: %systemd_user_postun_with_restart dockd.socket}

%files
%license LICENSE
%doc README.md
%{_bindir}/dockd
%{_bindir}/dockd-uninstall
%{_userunitdir}/dockd.service
%{_userunitdir}/dockd.socket
%{_datadir}/omarchy/plugins/org.ubermetroid.dockd


%changelog
* Tue Oct 06 2026 UberMetroid <ubermetroid@users.noreply.github.com> - 0.1.0-1
- Initial package release of dockd for Omarchy and Hyprland
