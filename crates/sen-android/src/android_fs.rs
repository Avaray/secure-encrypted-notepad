/// Android file system bridge for SAF (Storage Access Framework).
///
/// On Android, we cannot use filesystem paths directly (scoped storage).
/// Instead, file content is passed as raw byte arrays via JNI from
/// Kotlin's ContentResolver.
use std::sync::{Arc, Mutex};

/// Shared channel for receiving file data from the Kotlin side via JNI.
/// The Kotlin side reads the content:// URI into bytes and pushes them here,
/// then the Rust update loop picks them up.
#[derive(Default)]
pub struct FileChannel {
    /// Incoming file data from an Open operation (bytes + display name)
    pub pending_open: Option<(Vec<u8>, String)>,
    /// Incoming target URI for a Save operation
    pub pending_save_uri: Option<String>,
    /// Incoming keyfile data from a Select Keyfile operation (uri + bytes)
    pub pending_keyfile: Option<(String, Vec<u8>)>,
    /// Incoming directory URI from a Select Directory operation
    pub pending_directory_uri: Option<String>,
    /// Incoming confirmation from a Save operation (content:// URI was written successfully)
    pub pending_save_result: Option<bool>,
    /// Incoming biometric authentication result
    pub pending_biometric_result: Option<bool>,
    /// Incoming text input from the hidden EditText bridge
    pub pending_text_input: Vec<String>,
}

/// Global file channel instance shared between JNI callbacks and the Egui loop.
/// This uses lazy_static pattern via std::sync::OnceLock for thread safety.
static FILE_CHANNEL: std::sync::OnceLock<Arc<Mutex<FileChannel>>> = std::sync::OnceLock::new();

pub fn get_file_channel() -> Arc<Mutex<FileChannel>> {
    FILE_CHANNEL
        .get_or_init(|| Arc::new(Mutex::new(FileChannel::default())))
        .clone()
}

/// Called from JNI when the user has selected a file to open.
/// The Kotlin side reads the URI content into bytes and passes them here.
pub fn jni_deliver_open_file(data: Vec<u8>, name: String) {
    if let Ok(mut channel) = get_file_channel().lock() {
        channel.pending_open = Some((data, name));
    }
}

/// Called from JNI when the user has selected a location to save.
pub fn jni_deliver_save_uri(uri: String) {
    if let Ok(mut channel) = get_file_channel().lock() {
        channel.pending_save_uri = Some(uri);
    }
}

/// Called from JNI when the user has selected a keyfile.
pub fn jni_deliver_keyfile(uri: String, data: Vec<u8>) {
    if let Ok(mut channel) = get_file_channel().lock() {
        channel.pending_keyfile = Some((uri, data));
    }
}

/// Called from JNI when the user has selected a directory.
pub fn jni_deliver_directory_uri(uri: String) {
    if let Ok(mut channel) = get_file_channel().lock() {
        channel.pending_directory_uri = Some(uri);
    }
}

/// Called from JNI when a save operation completes.
pub fn jni_deliver_save_result(success: bool) {
    if let Ok(mut channel) = get_file_channel().lock() {
        channel.pending_save_result = Some(success);
    }
}

/// Called from JNI when a biometric authentication operation completes.
pub fn jni_deliver_biometric_result(success: bool) {
    if let Ok(mut channel) = get_file_channel().lock() {
        channel.pending_biometric_result = Some(success);
    }
}

/// Called from JNI when text is entered in the hidden EditText bridge.
pub fn jni_deliver_text_input(text: String) {
    if let Ok(mut channel) = get_file_channel().lock() {
        channel.pending_text_input.push(text);
    }
}
