//! The bundled Python standard library, embedded at build time by build.rs.
include!(concat!(env!("OUT_DIR"), "/stdlib.rs"));
