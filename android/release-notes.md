Android preview based on the exact source identified in the supplied working Aether APK, with the final SOCKS5 / HTTP CONNECT relay from desktop GUI v0.9.2.

Install the **universal-debug.apk** file on Android 8 or newer (ARM64 or ARMv7). The app installs alongside the original under the name **Aether Modified**.

In Advanced, enable **Use a final proxy**, enter the host, port and optional credentials, and select your working Aether protocol. Keep **Proxy mode** and **Split tunneling** off for whole-phone routing. Accept Android's VPN permission prompt. No Clash or browser proxy extension is needed.

Phone apps → Android VPN → Aether → final proxy → destination. SOCKS5 supports native UDP when both hops accept UDP ASSOCIATE. HTTP CONNECT carries TCP only. A failed final proxy does not fall back to another exit. Credentials stay in memory and must be entered again after restarting the app.

This is a debug-signed preview. CI checks include the v0.9.2 TCP/UDP relay tests, JNI lifecycle checks, Android compilation and lint, packaged native libraries, signatures and checksums. It still needs validation on a physical phone with your working proxy.

Source and build instructions: [android/README.md](https://github.com/dirtySnowman-I/Aether-modified/blob/android/android/README.md).
