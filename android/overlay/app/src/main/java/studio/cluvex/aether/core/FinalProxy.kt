package studio.cluvex.aether.core

import studio.cluvex.aether.model.FinalProxyProfile

/** In-process native relay. The final proxy is always reached THROUGH Aether. */
object FinalProxy {
    init { System.loadLibrary("aether_final_proxy") }
    @JvmStatic private external fun nativeStart(json: String, corePort: Int): Int
    @JvmStatic private external fun nativeVerify(port: Int): Boolean
    @JvmStatic private external fun nativeStop(port: Int)
    @JvmStatic private external fun nativeError(): String

    fun start(profile: FinalProxyProfile, corePort: Int): Int {
        val port = nativeStart(profile.toJson().toString(), corePort)
        check(port > 0) { nativeError() }
        return port
    }
    fun verify(port: Int) { check(nativeVerify(port)) { nativeError() } }
    fun stop(port: Int) { nativeStop(port) }
}

