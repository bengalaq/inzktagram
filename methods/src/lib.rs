//! Expone el ELF del guest (`FEED_GUEST_ELF`) y su image ID
//! (`FEED_GUEST_ID`). El image ID es el hash criptográfico del binario del
//! guest: cualquiera puede recompilarlo desde el código fuente y comprobar
//! que coincide, sin confiar en el servidor.

include!(concat!(env!("OUT_DIR"), "/methods.rs"));
