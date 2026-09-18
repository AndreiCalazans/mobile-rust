package com.example.mobilerust

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import uniffi.app_core.AppCore
import uniffi.app_core.AssetPrice
import uniffi.app_core.SessionState
import uniffi.app_core.WordEntry

// --- MVI state ------------------------------------------------------------

data class AppState(
    val session: SessionState = SessionState.LoggedOut,
    val words: List<WordEntry> = emptyList(),
    val bitcoin: AssetPrice? = null,
    val status: String = "",
)

// The ViewModel owns the single Rust AppCore. All logic is in Rust; this only
// dispatches intents and projects state.
class AppViewModel : ViewModel() {
    private val core = AppCore()  // Rust-backed handle
    private val _state = MutableStateFlow(AppState(session = core.currentSession()))
    val state: StateFlow<AppState> = _state

    fun login() {
        _state.value = _state.value.copy(session = core.login("Ada Lovelace"))
    }

    fun logout() {
        core.logout()
        _state.value = _state.value.copy(session = core.currentSession())
    }

    fun loadWords() = viewModelScope.launch {
        _state.value = _state.value.copy(status = "loading words…")
        val words = core.defineWords(listOf("serendipity", "rust", "mobile"))
        _state.value = _state.value.copy(words = words, status = "")
    }

    fun loadBitcoin() = viewModelScope.launch {
        _state.value = _state.value.copy(status = "loading BTC…")
        try {
            _state.value = _state.value.copy(bitcoin = core.fetchBitcoin(), status = "")
        } catch (e: Exception) {
            _state.value = _state.value.copy(status = "BTC error: ${e.message}")
        }
    }

    override fun onCleared() {
        core.close()  // free the Rust object deterministically
    }
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme {
                Surface(Modifier.fillMaxSize()) { AppScreen() }
            }
        }
    }
}

@Composable
fun AppScreen(vm: AppViewModel = viewModel()) {
    val state by vm.state.collectAsState()

    Column(
        Modifier.fillMaxSize().padding(24.dp).verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("Rust-core PoC", style = MaterialTheme.typography.headlineSmall)

        val who = when (val s = state.session) {
            is SessionState.Active -> "Signed in: ${s.user.displayName}"
            SessionState.LoggedOut -> "Signed out"
        }
        Text(who)

        when (state.session) {
            is SessionState.Active -> Button(onClick = vm::logout) { Text("Log out") }
            SessionState.LoggedOut -> Button(onClick = vm::login) { Text("Fake login") }
        }

        Button(onClick = vm::loadWords) { Text("Fetch words (REST)") }
        Button(onClick = vm::loadBitcoin) { Text("Fetch BTC (GraphQL)") }

        if (state.status.isNotEmpty()) Text(state.status)

        state.bitcoin?.let {
            Text("${it.symbol} (${it.name}) = \$${it.priceUsd}  day ${it.changeDayPercent}%")
        }

        state.words.forEach { w ->
            Text("• ${w.word} ${w.phonetic ?: ""} — ${w.definitions.firstOrNull()?.meaning ?: ""}")
        }
    }
}
