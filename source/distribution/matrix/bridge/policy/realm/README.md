# Native realm-setup codec

Start at [main.omg](main.omg): decode opaque host state, dispatch begin/next/
completion to [realm policy](../../../service/children/startup/realm/main.omg),
then encode the updated state. [encoding.omg](encoding.omg) owns native job and
command tags, ready/waiting representation and existing host status codes.

The adapter translates discovery details into named placement and realm-entry
facts. It does not decide whether to configure, start a picker or defer startup.
The service's setup state owns those choices and cleanup ordering; Rust retains
images, accepted connections, exact tasks, grants and the existing error paths.
