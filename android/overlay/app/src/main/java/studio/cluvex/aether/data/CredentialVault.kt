package studio.cluvex.aether.data

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import org.json.JSONObject
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * Encrypts the final proxy's username and password before they are written to
 * DataStore. The AES-256-GCM key lives in the Android Keystore: it never
 * leaves the device's secure storage and is removed with the app, so the
 * saved text is useless anywhere else. Any failure (a reset Keystore, a
 * damaged value) just means the user types the credentials again.
 */
object CredentialVault {
    private const val KEYSTORE = "AndroidKeyStore"
    private const val ALIAS = "aether_final_proxy_credentials"
    private const val TRANSFORMATION = "AES/GCM/NoPadding"
    private const val IV_BYTES = 12
    private const val TAG_BITS = 128

    @Synchronized
    private fun key(): SecretKey {
        val store = KeyStore.getInstance(KEYSTORE).apply { load(null) }
        (store.getKey(ALIAS, null) as? SecretKey)?.let { return it }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE)
        generator.init(
            KeyGenParameterSpec.Builder(
                ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build(),
        )
        return generator.generateKey()
    }

    /** Returns Base64(IV + ciphertext), or null if encryption is unavailable. */
    fun seal(username: String, password: String): String? = runCatching {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key())
        val plain = JSONObject().put("u", username).put("p", password)
            .toString().toByteArray(Charsets.UTF_8)
        Base64.encodeToString(cipher.iv + cipher.doFinal(plain), Base64.NO_WRAP)
    }.getOrNull()

    /** Returns (username, password), or null if the value cannot be opened. */
    fun open(sealed: String): Pair<String, String>? = runCatching {
        val bytes = Base64.decode(sealed, Base64.NO_WRAP)
        require(bytes.size > IV_BYTES)
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(TAG_BITS, bytes, 0, IV_BYTES))
        val json = JSONObject(
            String(cipher.doFinal(bytes, IV_BYTES, bytes.size - IV_BYTES), Charsets.UTF_8),
        )
        json.getString("u") to json.getString("p")
    }.getOrNull()
}
