package com.sen.android

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import android.util.Log
import android.text.Editable
import android.text.InputType
import android.text.TextWatcher
import android.view.ViewGroup
import android.view.WindowManager
import android.view.inputmethod.EditorInfo
import android.view.KeyEvent
import android.view.inputmethod.InputMethodManager
import android.widget.EditText
import android.widget.FrameLayout
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.documentfile.provider.DocumentFile
import com.google.androidgamesdk.GameActivity
import org.json.JSONArray
import org.json.JSONObject
import java.io.InputStream
import java.io.OutputStream
import java.util.concurrent.Executor

class MainActivity : GameActivity() {

    private lateinit var hiddenInput: EditText

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setupHiddenInput()
        Log.i("SEN", "MainActivity onCreate")
    }

    private fun setupHiddenInput() {
        runOnUiThread {
            hiddenInput = EditText(this).apply {
                inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_MULTI_LINE
                imeOptions = EditorInfo.IME_ACTION_NONE
                
                // Set listener for the Return key (Enter)
                setOnEditorActionListener { _, actionId, event ->
                    if (actionId == EditorInfo.IME_ACTION_DONE || 
                        actionId == EditorInfo.IME_ACTION_UNSPECIFIED ||
                        (event != null && event.keyCode == KeyEvent.KEYCODE_ENTER && event.action == KeyEvent.ACTION_DOWN)) {
                        nativeDeliverTextInput("\n")
                        true
                    } else {
                        false
                    }
                }

                layoutParams = ViewGroup.LayoutParams(1, 1)
                alpha = 0f
                setBackgroundColor(0)
                setText(" ")
                setSelection(1)

                addTextChangedListener(object : TextWatcher {
                    override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}
                    override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {
                        val currentText = s?.toString() ?: ""
                        if (currentText.isEmpty()) {
                            nativeDeliverTextInput("\u0008") // Backspace
                            setText(" ")
                            setSelection(1)
                        } else if (currentText.length > 1) {
                            val newText = currentText.substring(1)
                            nativeDeliverTextInput(newText)
                            setText(" ")
                            setSelection(1)
                        }
                    }
                    override fun afterTextChanged(s: Editable?) {}
                })
            }

            val frame = FrameLayout(this)
            frame.addView(hiddenInput)
            addContentView(frame, ViewGroup.LayoutParams(1, 1))
        }
    }

    // --- SAF Trigger Methods (Called from Rust) ---

    fun openFilePicker() {
        Log.i("SEN", "Opening File Picker")
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
        }
        startActivityForResult(intent, 1001)
    }

    fun saveFilePicker(suggestedName: String) {
        Log.i("SEN", "Opening Save Picker with suggested name: $suggestedName")
        val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(Intent.EXTRA_TITLE, suggestedName)
        }
        startActivityForResult(intent, 1002)
    }

    fun selectKeyfile() {
        Log.i("SEN", "Opening Keyfile Picker")
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
        }
        startActivityForResult(intent, 1003)
    }

    fun openDirectoryPicker() {
        Log.i("SEN", "Opening Directory Picker")
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        }
        startActivityForResult(intent, 1004)
    }

    // --- Activity Result Handling ---

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (resultCode == RESULT_OK && data != null) {
            val uri = data.data ?: return
            
            when (requestCode) {
                1001 -> handleOpenFile(uri)
                1002 -> handleSaveFile(uri)
                1003 -> handleKeyfile(uri)
                1004 -> handleDirectory(uri)
            }
        }
    }

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

    fun toggleKeyboard(show: Boolean) {
        runOnUiThread {
            val imm = getSystemService(INPUT_METHOD_SERVICE) as InputMethodManager
            if (show) {
                hiddenInput.requestFocus()
                imm.showSoftInput(hiddenInput, InputMethodManager.SHOW_IMPLICIT)
                Log.i("SEN", "Keyboard requested: SHOW (via HiddenInput)")
            } else {
                imm.hideSoftInputFromWindow(hiddenInput.windowToken, 0)
                Log.i("SEN", "Keyboard requested: HIDE")
            }
        }
    }

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
                        // We don't necessarily call Rust here, as user can try again
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

    fun readBytesFromUri(uriString: String): ByteArray? {
        return try {
            val uri = Uri.parse(uriString)
            contentResolver.openInputStream(uri)?.use { it.readBytes() }
        } catch (e: Exception) {
            Log.e("SEN", "Error reading bytes from URI $uriString: ${e.message}")
            null
        }
    }

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

    // --- Native Methods (Implemented in Rust) ---

    private external fun nativeDeliverOpenFile(data: ByteArray, name: String)
    private external fun nativeDeliverSaveUri(uriString: String)
    private external fun nativeDeliverKeyfile(uri: String, data: ByteArray)
    private external fun nativeDeliverDirectoryUri(uri: String)
    private external fun nativeDeliverTextInput(text: String)
    private external fun nativeDeliverBiometricResult(success: Boolean)
}
