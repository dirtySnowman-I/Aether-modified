package studio.cluvex.aether.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import studio.cluvex.aether.model.FinalProxyProfile
import studio.cluvex.aether.ui.components.DropdownSelector
import studio.cluvex.aether.ui.components.LtrOutlinedTextField

@Composable
fun FinalProxyPanel(value: FinalProxyProfile, onChange: (FinalProxyProfile) -> Unit, enabled: Boolean) {
    Text("Final proxy", style = MaterialTheme.typography.titleMedium)
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Text("Use a final proxy", Modifier.weight(1f))
        Switch(value.enabled, { onChange(value.copy(enabled = it)) }, enabled = enabled)
    }
    Text("Phone apps → Aether → final proxy. Websites see the final proxy's IP.", style = MaterialTheme.typography.bodySmall)
    if (!value.enabled) return
    Spacer(Modifier.height(12.dp))
    DropdownSelector(options = listOf("socks5", "http"), selected = value.kind,
        onSelect = { onChange(value.copy(kind = it, udpEnabled = it == "socks5" && value.udpEnabled)) },
        label = { if (it == "socks5") "SOCKS5" else "HTTP CONNECT" }, enabled = enabled)
    Spacer(Modifier.height(8.dp))
    LtrOutlinedTextField(value = value.host, onValueChange = { onChange(value.copy(host = it)) },
        label = { Text("Host") }, enabled = enabled, singleLine = true, modifier = Modifier.fillMaxWidth())
    LtrOutlinedTextField(value = value.port, onValueChange = { onChange(value.copy(port = it)) },
        label = { Text("Port") }, enabled = enabled, singleLine = true, modifier = Modifier.fillMaxWidth())
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Text("Username and password", Modifier.weight(1f))
        Switch(value.authenticate, { onChange(value.copy(authenticate = it)) }, enabled = enabled)
    }
    if (value.authenticate) {
        OutlinedTextField(value = value.username, onValueChange = { onChange(value.copy(username = it)) },
            label = { Text("Username") }, enabled = enabled, singleLine = true, modifier = Modifier.fillMaxWidth())
        OutlinedTextField(value = value.password, onValueChange = { onChange(value.copy(password = it)) },
            label = { Text("Password") }, visualTransformation = PasswordVisualTransformation(),
            enabled = enabled, singleLine = true, modifier = Modifier.fillMaxWidth())
        Text("Credentials stay in memory. Enter them again after restarting the app.", style = MaterialTheme.typography.bodySmall)
    }
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Text("Forward UDP", Modifier.weight(1f))
        Switch(value.udpEnabled, { onChange(value.copy(udpEnabled = it)) }, enabled = enabled && value.kind == "socks5")
    }
    Text("UDP requires SOCKS5 UDP ASSOCIATE at both hops. TCP-only mode uses mapped DNS for domain connections. ICMP is not supported.", style = MaterialTheme.typography.bodySmall)
    Text("A failed final proxy stops the connection. Split-tunneling exclusions still use normal networking.", style = MaterialTheme.typography.bodySmall)
}
