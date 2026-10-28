# Windows adapter

`build.omg` selects the checked `NativeWindowsUi` implementation and binds its
66 raw Win32 requirements to exact DLL exports for `windows_x86_64`. The portable
UI core does not own native handles, window policy or Windows resource backing.

`windows_host.omg` defines the public UI host contract and message/geometry data.
`windows_native.omg` implements that contract, including terminated-text checks,
message adaptation and conversions. It calls the raw requirements declared in
`win32_imports.omg`; those requirements acquire native candidates only through
this package's Build. Applications select the UI host without repeating DLL names.

Fonts, editing, event dispatch, timing and bitmap presentation remain in their
named sibling modules. The headless program exercises the portable core; the
Windows gallery and `tests/native_host` additionally require a Windows runner.
