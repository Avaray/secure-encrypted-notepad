package com.sen.android

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import android.util.Log
import android.view.KeyEvent
import android.view.WindowManager
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.Keep
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.documentfile.provider.DocumentFile
import com.google.androidgamesdk.GameActivity
import org.json.JSONArray
import org.json.JSONObject
import java.io.InputStream
import java.io.OutputStream

class MainActivity : GameActivity() {

    // --- SAF Activity Result Launchers ---
    
    private val openFileLauncher = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        if (result.resultCode == RESULT_OK) {
            result.data?.data?.let { handleOpenFile(it) }
        }
    }

    private val saveFileLauncher = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        if (result.resultCode == RESULT_OK) {
            result.data?.data?.let { handleSaveFile(it) }
        }
    }

    private val keyfileLauncher = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        if (result.resultCode == RESULT_OK) {
            result.data?.data?.let { handleKeyfile(it) }
        }
    }

    private val directoryLauncher = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        if (result.resultCode == RESULT_OK) {
            result.data?.data?.let { handleDirectory(it) }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.i("SEN", "MainActivity onCreate")
    }

    override fun onPause() {
        super.onPause()
        Log.i("SEN", "MainActivity onPause")
        nativeAppPaused()
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        try {
            if (event.action == KeyEvent.ACTION_DOWN) {
                when (event.keyCode) {
                    KeyEvent.KEYCODE_DEL,
                    KeyEvent.KEYCODE_FORWARD_DEL,
                    KeyEvent.KEYCODE_ENTER,
                    KeyEvent.KEYCODE_NUMPAD_ENTER -> {
                        nativeDeliverSpecialKey(event.keyCode)
                    }
                }
            }

            if (event.action == KeyEvent.ACTION_MULTIPLE && event.keyCode == KeyEvent.KEYCODE_UNKNOWN) {
                val chars = event.characters
                if (!chars.isNullOrEmpty()) {
                    nativeDeliverTypedText(chars)
                }
            } else if (event.action == KeyEvent.ACTION_DOWN) {
                val unicode = event.unicodeChar
                if (unicode > 0 && !event.isCtrlPressed && !event.isAltPressed && !event.isMetaPressed) {
                    nativeDeliverTypedText(String(Character.toChars(unicode)))
                }
            }
        } catch (e: Exception) {
            Log.w("SEN", "dispatchKeyEvent text bridge error: ${e.message}")
        }
        return super.dispatchKeyEvent(event)
    }

    // --- SAF Trigger Methods (Called from Rust) ---

    @Keep
    fun openFilePicker() {
        Log.i("SEN", "Opening File Picker")
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
        }
        openFileLauncher.launch(intent)
    }

    @Keep
    fun saveFilePicker(suggestedName: String) {
        Log.i("SEN", "Opening Save Picker with suggested name: $suggestedName")
        val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(Intent.EXTRA_TITLE, suggestedName)
        }
        saveFileLauncher.launch(intent)
    }

    @Keep
    fun selectKeyfile() {
        Log.i("SEN", "Opening Keyfile Picker")
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
        }
        keyfileLauncher.launch(intent)
    }

    @Keep
    fun openDirectoryPicker() {
        Log.i("SEN", "Opening Directory Picker")
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        }
        directoryLauncher.launch(intent)
    }

    // --- Activity Result Handlers ---

    private fun handleOpenFile(uri: Uri) {
        try {
            val contentResolver = applicationContext.contentResolver
            val inputStream: InputStream? = contentResolver.openInputStream(uri)
            val bytes = inputStream?.readBytes() ?: return
            val fileName = getFileName(uri)
            
            nativeDeliverOpenFile(bytes, fileName)
            Log.i("SEN", "Delivered file to Rust: $fileName")
        } catch (e: Exception) {
            Log.e("SEN", "Error reading file: ${e.message}")
        }
    }

    private fun handleSaveFile(uri: Uri) {
        Log.i("SEN", "Save target URI ready: $uri")
        nativeDeliverSaveUri(uri.toString())
    }

    private fun handleKeyfile(uri: Uri) {
        try {
            val contentResolver = applicationContext.contentResolver
            val takeFlags: Int = Intent.FLAG_GRANT_READ_URI_PERMISSION
            contentResolver.takePersistableUriPermission(uri, takeFlags)

            val inputStream: InputStream? = contentResolver.openInputStream(uri)
            val bytes = inputStream?.readBytes() ?: return
            
            nativeDeliverKeyfile(uri.toString(), bytes)
            Log.i("SEN", "Delivered keyfile to Rust (persistent): $uri")
        } catch (e: Exception) {
            Log.e("SEN", "Error reading keyfile: ${e.message}")
        }
    }

    private fun handleDirectory(uri: Uri) {
        try {
            val contentResolver = applicationContext.contentResolver
            val takeFlags: Int = Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            contentResolver.takePersistableUriPermission(uri, takeFlags)
            
            Log.i("SEN", "Selected directory: $uri")
            nativeDeliverDirectoryUri(uri.toString())
        } catch (e: Exception) {
            Log.e("SEN", "Error taking directory permission: ${e.message}")
        }
    }

    private fun getFileName(uri: Uri): String {
        var name = "unknown.sen"
        contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (cursor.moveToFirst()) {
                name = cursor.getString(nameIndex)
            }
        }
        return name
    }

    // --- Helper for Rust (Called from Rust via JNI) ---

    @Keep
    fun writeToFileUri(uriString: String, data: ByteArray): Boolean {
        return try {
            val uri = Uri.parse(uriString)
            val outputStream: OutputStream? = contentResolver.openOutputStream(uri)
            outputStream?.use { it.write(data) }
            Log.i("SEN", "Successfully wrote ${data.size} bytes to $uriString")
            true
        } catch (e: Exception) {
            Log.e("SEN", "Failed to write to URI: ${e.message}")
            false
        }
    }

    @Keep
    fun setScreenCaptureProtection(enabled: Boolean) {
        runOnUiThread {
            if (enabled) {
                window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
                Log.i("SEN", "Screen capture protection ENABLED")
            } else {
                window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
                Log.i("SEN", "Screen capture protection DISABLED")
            }
        }
    }

    @Keep
    fun showBiometricPrompt() {
        runOnUiThread {
            val executor = ContextCompat.getMainExecutor(this)
            val biometricPrompt = BiometricPrompt(this, executor,
                object : BiometricPrompt.AuthenticationCallback() {
                    override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                        super.onAuthenticationError(errorCode, errString)
                        Log.e("SEN", "Authentication error: $errString")
                        nativeDeliverBiometricResult(false)
                    }

                    override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                        super.onAuthenticationSucceeded(result)
                        Log.i("SEN", "Authentication succeeded!")
                        nativeDeliverBiometricResult(true)
                    }

                    override fun onAuthenticationFailed() {
                        super.onAuthenticationFailed()
                        Log.w("SEN", "Authentication failed")
                    }
                })

            val promptInfo = BiometricPrompt.PromptInfo.Builder()
                .setTitle("Unlock Secure Notepad")
                .setSubtitle("Use your biometric credential to unlock")
                .setNegativeButtonText("Cancel")
                .build()

            biometricPrompt.authenticate(promptInfo)
        }
    }

    @Keep
    fun listDirectory(uriString: String): String {
        return try {
            val treeUri = Uri.parse(uriString)
            Log.i("SEN", "Listing directory for URI: $treeUri")
            
            val docFile = DocumentFile.fromTreeUri(applicationContext, treeUri)
            val jsonArray = JSONArray()
            
            if (docFile != null && docFile.isDirectory) {
                val files = docFile.listFiles()
                Log.i("SEN", "Found ${files.size} files in directory")
                
                files.forEach { file ->
                    val obj = JSONObject()
                    obj.put("uri", file.uri.toString())
                    obj.put("name", file.name ?: "unknown")
                    obj.put("is_dir", file.isDirectory)
                    obj.put("is_expanded", false)
                    obj.put("depth", 0)
                    jsonArray.put(obj)
                }
            } else {
                Log.w("SEN", "docFile is null or not a directory: $docFile")
            }
            jsonArray.toString()
        } catch (e: Exception) {
            Log.e("SEN", "Error listing directory: ${e.message}")
            "[]"
        }
    }

    @Keep
    fun readBytesFromUri(uriString: String): ByteArray? {
        return try {
            val uri = Uri.parse(uriString)
            contentResolver.openInputStream(uri)?.use { it.readBytes() }
        } catch (e: Exception) {
            Log.e("SEN", "Error reading bytes from URI $uriString: ${e.message}")
            null
        }
    }

    @Keep
    fun getUriMetadata(uriString: String): String {
        return try {
            val uri = Uri.parse(uriString)
            val obj = JSONObject()
            contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
                if (cursor.moveToFirst()) {
                    obj.put("name", cursor.getString(nameIndex) ?: "unknown")
                    obj.put("size", cursor.getLong(sizeIndex))
                }
            }
            obj.toString()
        } catch (e: Exception) {
             "{ \"error\": \"${e.message}\" }"
        }
    }

    @Keep
    fun getScreenDensity(): Float {
        return resources.displayMetrics.density
    }

    // --- Native Methods (Implemented in Rust) ---

    private external fun nativeDeliverOpenFile(data: ByteArray, name: String)
    private external fun nativeDeliverSaveUri(uriString: String)
    private external fun nativeDeliverKeyfile(uri: String, data: ByteArray)
    private external fun nativeDeliverDirectoryUri(uri: String)
    private external fun nativeDeliverBiometricResult(success: Boolean)
    private external fun nativeAppPaused()
    private external fun nativeDeliverTypedText(text: String)
    private external fun nativeDeliverSpecialKey(keyCode: Int)
}
