// Global and static variables are heavily discouraged in many languages.
// The use of them in this project is strictly limited to:
//
// * situations where we don't have any alternative
// * situations where impact on mock-ability is really zero,
//     and there are clear maintenance/performance benefits of doing so

use chacha20poly1305::XChaCha20Poly1305;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CIPHER: XChaCha20Poly1305 =
        crate::plugin_auth_crypto::generate_xchacha20poly1305_cipher();
}
