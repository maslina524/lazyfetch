# src/windows/

Implementation of OS-dependent functions for Windows

This is one of the directories responsible for implementing OS-dependent functions; in `main.rs`, depending on the OS, one of the directories becomes imp at build time — it is used for multiplatform access to the OS, for this all directories and their files must have the same API.