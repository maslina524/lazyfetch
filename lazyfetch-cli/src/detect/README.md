# src/detect/

Here are the implementation files for modules that have complex logic or require different implementations for different systems

Each directory and file located in the root of this directory is the name of a module, which contains the implementation; if it is a file, the implementation is multiplatform; if it is a directory, it contains files named after the OS for which the module is implemented in this file (`windows`, `linux`, `android`) and `mod.rs` in which, at compile time, it is determined which of the files will be compiled for a specific OS and, sometimes, the *Info structure, which is assembled in detect, is then used to fill the module's fields.