package studio.cluvex.aether.data

import android.content.Context
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import studio.cluvex.aether.model.FinalProxyProfile
import studio.cluvex.aether.model.ConnectionProfile
import studio.cluvex.aether.model.EndpointMode
import studio.cluvex.aether.model.IpVersion
import studio.cluvex.aether.model.Noize
import studio.cluvex.aether.model.Protocol
import studio.cluvex.aether.model.ScanMode
import studio.cluvex.aether.model.SplitMode

private val Context.dataStore by preferencesDataStore(name = "aether_profile")

/** Persists the last-used [ConnectionProfile] with Jetpack DataStore. */
class ProfileStore(private val context: Context) {
    private object Keys {
        val finalEnabled = booleanPreferencesKey("finalEnabled")
        val finalKind = stringPreferencesKey("finalKind")
        val finalHost = stringPreferencesKey("finalHost")
        val finalPort = stringPreferencesKey("finalPort")
        val finalAuth = booleanPreferencesKey("finalAuth")
        val finalUdp = booleanPreferencesKey("finalUdp")
        val finalRemember = booleanPreferencesKey("finalRemember")
        /** Username and password, encrypted by [CredentialVault]. */
        val finalSecret = stringPreferencesKey("finalSecret")

        val protocol = stringPreferencesKey("protocol")
        val scan = stringPreferencesKey("scan")
        val ip = stringPreferencesKey("ip")
        val quick = booleanPreferencesKey("quick")
        val h2 = booleanPreferencesKey("h2")
        val share = booleanPreferencesKey("share")
        // Added in 1.2.0
        val noize = stringPreferencesKey("noize")
        val endpoint = stringPreferencesKey("endpoint")
        val peer = stringPreferencesKey("peer")
        val range = stringPreferencesKey("range")
        val keepalive = intPreferencesKey("keepalive")
        val fragment = booleanPreferencesKey("fragment")
        val ech = booleanPreferencesKey("ech")
        val mtu = intPreferencesKey("mtu")
        val proxy = booleanPreferencesKey("proxy")
        val split = stringPreferencesKey("split")
        val splitApps = stringPreferencesKey("splitApps")
    }

    /** Last pair written, so typing elsewhere does not re-encrypt each time. */
    @Volatile private var lastSealed: Pair<String, String>? = null

    val profile: Flow<ConnectionProfile> = context.dataStore.data.map { prefs ->
        val d = ConnectionProfile()
        val remember = prefs[Keys.finalRemember] ?: true
        val saved = if (remember) prefs[Keys.finalSecret]?.let(CredentialVault::open) else null
        if (saved != null) lastSealed = saved
        ConnectionProfile(
            finalProxy = FinalProxyProfile(
                enabled = prefs[Keys.finalEnabled] ?: false,
                kind = prefs[Keys.finalKind] ?: "socks5",
                host = prefs[Keys.finalHost] ?: "",
                port = prefs[Keys.finalPort] ?: "1080",
                authenticate = prefs[Keys.finalAuth] ?: false,
                udpEnabled = prefs[Keys.finalUdp] ?: true,
                username = saved?.first ?: "",
                password = saved?.second ?: "",
                rememberCredentials = remember,
            ),
            protocol = prefs[Keys.protocol]
                ?.let { runCatching { Protocol.valueOf(it) }.getOrNull() } ?: Protocol.AUTO,
            scanMode = prefs[Keys.scan]
                ?.let { runCatching { ScanMode.valueOf(it) }.getOrNull() } ?: ScanMode.BALANCED,
            ipVersion = prefs[Keys.ip]
                ?.let { runCatching { IpVersion.valueOf(it) }.getOrNull() } ?: IpVersion.V4,
            quickReconnect = prefs[Keys.quick] ?: true,
            masqueHttp2 = prefs[Keys.h2] ?: false,
            lanShare = prefs[Keys.share] ?: false,
            noize = prefs[Keys.noize]
                ?.let { runCatching { Noize.valueOf(it) }.getOrNull() } ?: Noize.OFF,
            endpointMode = prefs[Keys.endpoint]
                ?.let { runCatching { EndpointMode.valueOf(it) }.getOrNull() } ?: EndpointMode.AUTO,
            manualPeer = prefs[Keys.peer] ?: "",
            manualRange = prefs[Keys.range] ?: "",
            keepalive = prefs[Keys.keepalive] ?: 0,
            fragment = prefs[Keys.fragment] ?: false,
            ech = prefs[Keys.ech] ?: false,
            mtu = prefs[Keys.mtu] ?: d.mtu,
            proxyMode = prefs[Keys.proxy] ?: false,
            splitMode = prefs[Keys.split]
                ?.let { runCatching { SplitMode.valueOf(it) }.getOrNull() } ?: SplitMode.OFF,
            splitApps = prefs[Keys.splitApps]
                ?.split(",")?.map { it.trim() }?.filter { it.isNotEmpty() } ?: emptyList(),
        )
    }.flowOn(Dispatchers.IO) // Keystore decryption stays off the main thread.

    suspend fun save(profile: ConnectionProfile) {
        val proxy = profile.finalProxy
        val pair = proxy.username to proxy.password
        // null = leave the stored secret as it is; "" = delete it.
        val secret: String? = withContext(Dispatchers.IO) {
            when {
                !proxy.rememberCredentials -> ""
                // Authentication off for now: keep what was saved so turning
                // it back on does not ask again.
                !proxy.authenticate -> null
                pair.first.isEmpty() && pair.second.isEmpty() -> ""
                pair == lastSealed -> null
                else -> CredentialVault.seal(pair.first, pair.second)
            }
        }
        context.dataStore.edit { prefs ->
            val final = proxy.withoutSecrets()
            prefs[Keys.finalEnabled] = final.enabled
            prefs[Keys.finalKind] = final.kind
            prefs[Keys.finalHost] = final.host
            prefs[Keys.finalPort] = final.port
            prefs[Keys.finalAuth] = final.authenticate
            prefs[Keys.finalUdp] = final.udpEnabled
            prefs[Keys.finalRemember] = final.rememberCredentials
            if (secret != null) {
                if (secret.isEmpty()) {
                    prefs.remove(Keys.finalSecret)
                } else {
                    prefs[Keys.finalSecret] = secret
                }
            }

            prefs[Keys.protocol] = profile.protocol.name
            prefs[Keys.scan] = profile.scanMode.name
            prefs[Keys.ip] = profile.ipVersion.name
            prefs[Keys.quick] = profile.quickReconnect
            prefs[Keys.h2] = profile.masqueHttp2
            prefs[Keys.share] = profile.lanShare
            prefs[Keys.noize] = profile.noize.name
            prefs[Keys.endpoint] = profile.endpointMode.name
            prefs[Keys.peer] = profile.manualPeer
            prefs[Keys.range] = profile.manualRange
            prefs[Keys.keepalive] = profile.keepalive
            prefs[Keys.fragment] = profile.fragment
            prefs[Keys.ech] = profile.ech
            prefs[Keys.mtu] = profile.mtu
            prefs[Keys.proxy] = profile.proxyMode
            prefs[Keys.split] = profile.splitMode.name
            prefs[Keys.splitApps] = profile.splitApps.joinToString(",")
        }
        if (secret != null) lastSealed = if (secret.isEmpty()) null else pair
    }
}

