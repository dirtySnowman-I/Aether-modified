# Aether Modified for Android

Android client with the final SOCKS5 / HTTP CONNECT proxy chain from desktop
Aether-GUI **v0.9.2**. The app captures phone traffic itself; no Clash or browser
extension is required.

## Source and version

The uploaded `base.apk` identifies its build revision as
`c02509ebe2df6cf602cb60a59be18a8fb3d49509`. That exact revision exists in
[QW-AI-Code/Aether](https://github.com/QW-AI-Code/Aether/tree/c02509ebe2df6cf602cb60a59be18a8fb3d49509).
Its native VPN, TUN addresses, in-process hev bridge, Compose interface, split
tunneling, protocol controls, and diagnostics are reused.

The Android build checks out that source and applies `overlay/`. The existing
desktop code and `src-tauri/exit-proxy` are based on GUI v0.9.2 commit
`cd620845bcc8c8713ce56a9e37900dd6bbc69dbf`; the relay is used unchanged through
`exit-proxy-jni`. Aether core v1.5.0 is pinned to
`66a798b7771d5ffbb28fc858bffc99fb67295baf`. The phone traffic bridge is rebuilt
from hev 2.18.0, with 16 KB ELF alignment.

The modified app uses package `studio.aether.modified` and installs alongside
the original app. Only one Android VPN can be active at a time. APKs are
preview/debug signed. The original app's signing key is not used.

## Use

1. Download the `Aether-Modified-Android-0.9.2` artifact from a successful
   **Android final proxy** workflow run. Unzip it and install the universal APK
   (Android 8.0+, ARM64 or ARMv7).
2. Open **Aether Modified**, open Advanced, and choose your working protocol
   and settings. WireGuard remains available.
3. Enable **Use a final proxy** and enter the proxy host, port, and optional
   username/password. Enable **Forward UDP** for a SOCKS5 proxy that supports
   UDP ASSOCIATE. Credentials are held in memory and must be entered again
   after restarting the app.
4. Leave **Proxy mode** off for whole-phone routing. Leave split tunneling off
   to capture every other app. Press Connect and accept Android's VPN prompt.
5. The app checks both proxy hops and then runs its end-to-end connectivity
   diagnostics through the final proxy. Verify your public IP in a browser.

Traffic path: phone app → Android VPN → hev → local final-proxy relay → Aether
WireGuard/MASQUE tunnel → your final proxy → destination.

Native UDP uses nested SOCKS5 UDP envelopes; UDP is not wrapped in TCP. Both
Aether and the final SOCKS5 server must accept UDP ASSOCIATE. HTTP CONNECT
carries TCP only. In TCP-only mode the VPN uses hev mapped DNS so TCP domain
connections can resolve at the final proxy; other UDP traffic is rejected.
SOCKS5 does not carry ICMP ping. Apps explicitly excluded by split tunneling
use normal networking. The app's own transport UID is excluded to prevent a
recursive VPN loop, matching the supplied Android app's implementation.

A final proxy error never falls back to a direct connection or to Aether's
exit. Disconnect restores normal Android networking; this is not an always-on
VPN or a kill switch. Reconnect rebuilds and validates the entire chain,
including fresh UDP associations.

## Build

The workflow pins the Android source, core, and hev revisions; it does not run
the upstream auto-upgrade or publishing workflow. `overlay/` is the full set
of client changes. `prepare-build.py` changes the app identity, adds an Android
parent-pipe lifecycle guard to the core, and aligns native libraries.

Build prerequisites: Java 17, Gradle 8.9, Android SDK 35, NDK 28.2.13676358,
Rust with Android targets, cargo-ndk, Go, CMake, and Clang. The workflow tests
the desktop relay and JNI configuration, builds both CPU variants, runs
Android lint, verifies native contents/alignment and APK signatures, and
publishes checksums with the APK artifacts.

Source licenses and credits remain with Aether-GUI, QW-AI-Code/Aether,
CluvexStudio/Aether, and heiher/hev-socks5-tunnel. The modified source is
published under this repository's AGPL-3.0 license.
