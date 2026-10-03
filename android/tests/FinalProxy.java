package studio.cluvex.aether.core;

import java.net.ServerSocket;

/** Exercises the real JNI entry points, not a mock binding. */
public final class FinalProxy {
    static { System.loadLibrary("aether_final_proxy"); }
    private static native int nativeStart(String json, int corePort);
    private static native boolean nativeVerify(int port);
    private static native void nativeStop(int port);
    private static native String nativeError();

    private static void check(boolean ok, String message) {
        if (!ok) throw new AssertionError(message);
    }

    public static void main(String[] args) throws Exception {
        check(nativeStart("malformed-private-password", 1819) == 0, "Bad JSON accepted");
        check(!nativeError().contains("private-password"), "Input leaked into error");
        String config = "{\"kind\":\"socks5\",\"host\":\"proxy.test\",\"port\":1080,\"udp_enabled\":true}";
        int corePort;
        try (ServerSocket unused = new ServerSocket(0)) { corePort = unused.getLocalPort(); }
        int port = nativeStart(config, corePort);
        check(port > 0, "Cannot start relay: " + nativeError());
        check(nativeStart(config, corePort) == 0, "Duplicate session accepted");
        check(!nativeVerify(port), "Unreachable core passed verification");
        nativeStop(port);
        try (ServerSocket released = new ServerSocket(port)) {
            check(released.getLocalPort() == port, "Listener remained open after stop");
        }
        int next = nativeStart(config, corePort);
        check(next > 0, "Cannot reconnect: " + nativeError());
        nativeStop(next);
        System.out.println("JNI configuration, verification, disconnect and reconnect passed");
    }
}
