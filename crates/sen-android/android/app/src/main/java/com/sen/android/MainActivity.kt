package com.sen.android

import android.content.Intent
import android.os.Bundle
import android.util.Log
import com.google.androidgamesdk.GameActivity

import android.view.View
import android.os.Build
import android.view.WindowInsets
import android.view.WindowInsetsController

class MainActivity : GameActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Hide the status bar and navigation bar to make the app immersive (fullscreen)
        // This prevents the system UI from intercepting touches on our Egui toolbar
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            window.insetsController?.let {
                it.hide(WindowInsets.Type.statusBars() or WindowInsets.Type.navigationBars())
                it.systemBarsBehavior = WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            }
        } else {
            @Suppress("DEPRECATION")
            window.decorView.systemUiVisibility = (View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                    or View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                    or View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                    or View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                    or View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                    or View.SYSTEM_UI_FLAG_FULLSCREEN)
        }
        
        Log.i("SEN", "MainActivity onCreate - Android immersive mode enabled for GameActivity")
    }

    // Example handler for file picking that Rust can call via JNI
    fun openFilePicker() {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*" // SEN or text files
        }
        startActivityForResult(intent, 1001)
    }

    fun saveFilePicker(suggestedName: String) {
        val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(Intent.EXTRA_TITLE, suggestedName)
        }
        startActivityForResult(intent, 1002)
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (resultCode == RESULT_OK && data != null) {
            val uri = data.data ?: return
            
            // Log the URI for now. 
            // In a complete implementation, this URI string is passed down to Rust via JNI.
            when (requestCode) {
                1001 -> Log.i("SEN", "Opened URI: \$uri")
                1002 -> Log.i("SEN", "Saved to URI: \$uri")
            }
        }
    }
}
