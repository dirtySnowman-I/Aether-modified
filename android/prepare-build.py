"""Apply build identity and lifecycle changes to the pinned Android source."""
from pathlib import Path
import sys

root = Path(sys.argv[1])
p = root / 'app/build.gradle.kts'
s = p.read_text()
assert 'versionName = "1.2.2"' in s
s = s.replace('applicationId = "studio.cluvex.aether"', 'applicationId = "studio.aether.modified"')
s = s.replace('versionCode = 6', 'versionCode = 1')
s = s.replace('versionName = "1.2.2"', 'versionName = "0.9.2-android.1"')
s = s.replace('useLegacyPackaging = true', 'useLegacyPackaging = true\n            keepDebugSymbols += setOf("**/libaether.so")')
# A separate package installs alongside the original APK. Debug preview signing
# avoids using the upstream maintainer's public CI signing key.
p.write_text(s)
p = root / 'app/src/main/res/values/strings.xml'
s = p.read_text().replace('<string name="app_name">Aether</string>', '<string name="app_name">Aether Modified</string>')
p.write_text(s)
p = root / 'app/src/main/res/values-fa/strings.xml'
if p.exists():
    import re
    p.write_text(re.sub(r'(<string name="app_name">).*?(</string>)', r'\1Aether Modified\2', p.read_text()))
p = root / 'app/src/main/AndroidManifest.xml'
s = p.read_text().replace('<intent-filter>\n                <action android:name="android.net.VpnService" />', '<meta-data android:name="android.net.VpnService.SUPPORTS_ALWAYS_ON" android:value="false" />\n            <intent-filter>\n                <action android:name="android.net.VpnService" />')
p.write_text(s)
# This source-compatible core is the exact v1.5.0 used by desktop GUI v0.9.2.
# A monitor on the private parent pipe prevents orphan engines after Android
# kills the app process. No credential or command is read from this pipe.
p = root / '.native/aether/aether/src/main.rs'
s = p.read_text()
needle = 'async fn main() -> Result<()> {'
assert s.count(needle) == 1
s = s.replace(needle, needle + '''
    #[cfg(target_os = "android")]
    if std::env::var("AETHER_ANDROID_PARENT_PIPE").as_deref() == Ok("1") {
        std::thread::spawn(|| {
            use std::io::Read;
            let mut byte = [0u8; 1];
            loop {
                match std::io::stdin().read(&mut byte) {
                    Ok(0) | Err(_) => std::process::exit(0),
                    Ok(_) => {},
                }
            }
        });
    }
''')
p.write_text(s)
# Build hev's C bridge with 16 KB page alignment on newer Android devices.
p = root / 'scripts/build-natives.sh'
s = p.read_text().replace('"APP_CFLAGS=-O3"', '"APP_CFLAGS=-O3" \\\n      "APP_LDFLAGS=-Wl,-z,max-page-size=16384"')
s = s.replace(' -shared ', ' -Wl,-z,max-page-size=16384 -shared ')
p.write_text(s)

