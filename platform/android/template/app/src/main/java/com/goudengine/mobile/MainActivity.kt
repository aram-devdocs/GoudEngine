package com.goudengine.mobile

import android.os.Bundle
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.goudengine.core.GoudEngine

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        GoudEngine.ensureLoaded()

        val label = TextView(this).apply {
            text = "GoudEngine Android template"
            textSize = 18f
            setPadding(32, 48, 32, 48)
        }

        setContentView(label)
    }
}
