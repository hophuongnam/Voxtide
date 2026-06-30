package com.voxtide.desktop

import android.Manifest
import android.content.pm.PackageManager
import android.os.Bundle
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  private var mainWebView: WebView? = null

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // RECORD_AUDIO is needed for live capture (cpal Path A or WebView getUserMedia
    // Path B). Request up front; the Phase 0.6 spike decides which path consumes it.
    if (checkSelfPermission(Manifest.permission.RECORD_AUDIO)
        != PackageManager.PERMISSION_GRANTED) {
      requestPermissions(arrayOf(Manifest.permission.RECORD_AUDIO), 1)
    }
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    mainWebView = webView
  }

  override fun onStop() {
    emitAndroidStop()
    super.onStop()
  }

  override fun onDestroy() {
    emitAndroidStop()
    mainWebView = null
    super.onDestroy()
  }

  private fun emitAndroidStop() {
    mainWebView?.post {
      mainWebView?.evaluateJavascript(
        "window.dispatchEvent(new Event('voxtide:android-stop'))",
        null
      )
    }
  }
}
