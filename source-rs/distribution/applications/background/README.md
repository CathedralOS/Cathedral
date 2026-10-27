# Background executable adapter

[main.rs](main.rs) declares the interaction import and starts the common Omega
host. [build.rs](build.rs) packages the independently compiled export;
[native.rs](native.rs) binds it into the task. The app's sequence and rendering
live in [Omega](../../../../source/distribution/applications/background/README.md).

`cargo test --locked -p cathedral-background --lib` exercises the real compiled
export on Windows. The `startup-fault` feature retains the existing deliberate
panic used to test the owner's bounded background restart policy.
